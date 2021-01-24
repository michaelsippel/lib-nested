use {
    async_std::stream::StreamExt,
    std::{
        sync::{Arc, Weak, RwLock},
        collections::{HashMap, HashSet}
    },
    crate::{
        core::{
            View, Observer, ObserverExt, ObserverBroadcast,
            ViewPort, InnerViewPort, OuterViewPort,
            channel::{ChannelSender, ChannelReceiver}
        },
        sequence::SequenceView
    }
};

impl<V1, V2> OuterViewPort<V1>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    pub fn flatten(&self) -> OuterViewPort<dyn SequenceView<Item = V2::Item>> {
        let port = ViewPort::new();
        Flatten::new(self.clone(), port.inner());
        port.into_outer()
    }
}

pub struct Flatten<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    length: usize,
    top: Arc<RwLock<TopObserver<V1, V2>>>,
    chunks: HashMap<usize, Arc<RwLock<BotObserver<V2>>>>,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = V2::Item>>>>
}

struct TopObserver<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    view: Option<Arc<V1>>,
    sender: ChannelSender<HashSet<usize>>
}

struct BotObserver<V2>
where V2: SequenceView + ?Sized + 'static
{
    offset: usize,
    view: Option<Arc<V2>>,
    sender: ChannelSender<HashSet<usize>>
}

impl<V1, V2> Observer<V1> for TopObserver<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    fn reset(&mut self, view: Option<Arc<V1>>) {
        let old_len = self.view.len().unwrap_or(0);
        self.view = view;
        let new_len = self.view.len().unwrap_or(0);

        self.notify_each(0 .. std::cmp::max(old_len, new_len));
    }

    fn notify(&self, chunk_idx: &usize) {
        self.sender.send(*chunk_idx);
    }
}

impl<V2> Observer<V2> for BotObserver<V2>
where V2: SequenceView + ?Sized + 'static
{
    fn reset(&mut self, src: Option<Arc<V2>>) {
        let old_len = self.view.len().unwrap_or(0);
        self.view = src;
        let new_len = self.view.len().unwrap_or(0);

        self.notify_each(0 .. std::cmp::max(old_len, new_len));
    }

    fn notify(&self, idx: &usize) {
        self.sender.send(*idx);
    }
}

impl<V1, V2> View for Flatten<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    type Msg = usize;
}

impl<V1, V2> SequenceView for Flatten<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    type Item = V2::Item;

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let chunk = self.chunks[&self.get_chunk_idx(*idx)?].read().unwrap();
        chunk.view.get(&(*idx - chunk.offset))
    }

    fn len(&self) -> Option<usize> {
        Some(self.length)
    }
}

impl<V1, V2> Flatten<V1, V2>
where V1: SequenceView<Item = OuterViewPort<V2>> + ?Sized + 'static,
      V2: SequenceView + ?Sized + 'static
{
    pub fn new(
        top_port: OuterViewPort<V1>,
        out_port: InnerViewPort<dyn SequenceView<Item = V2::Item>>
    ) -> Arc<RwLock<Self>> {
        let (sender, mut recv) = crate::core::channel::set_channel();

        let top_obs = Arc::new(RwLock::new(
            TopObserver {
                view: None,
                sender
            }
        ));

        let flat = Arc::new(RwLock::new(Flatten::<V1, V2> {
            length: 0,
            top: top_obs.clone(),
            chunks: HashMap::new(),
            cast: out_port.get_broadcast()
        }));

        let f = flat.clone();
        let cast = out_port.get_broadcast();
        async_std::task::spawn(async move {
            while let Some(chunk_idx) = recv.next().await {
                if let Some(mut chunk_rcv) = f.write().unwrap().update_chunk(chunk_idx) {
                    let f = f.clone();
                    let cast = cast.clone();
                    async_std::task::spawn(async move {
                        while let Some(idx) = chunk_rcv.next().await {
                            let mut flat = f.write().unwrap();

                            let chunk = flat.chunks[&chunk_idx].read().unwrap();
                            let chunk_offset = chunk.offset;
                            let chunk_len = chunk.view.len().unwrap_or(0);
                            drop(chunk);

                            let mut dirty_idx = Vec::new();
                            if idx+1 >= chunk_len {
                                dirty_idx = flat.update_offsets(chunk_idx);
                            }

                            drop(flat);

                            cast.notify(&(idx + chunk_offset));
                            cast.notify_each(dirty_idx);
                        }
                    });
                }
            }
        });

        top_port.add_observer(top_obs);

        out_port.set_view(Some(flat.clone()));
        flat
    }

    /// the top-sequence has changed the item at chunk_idx,
    /// create a new observer for the contained sub sequence
    fn update_chunk(&mut self, chunk_idx: usize) -> Option<ChannelReceiver<HashSet<usize>>> {
        if let Some(chunk_port) = self.top.read().unwrap().view.get(&chunk_idx) {
            let (sender, recv) = crate::core::channel::set_channel();
            let chunk_obs = Arc::new(RwLock::new(
                BotObserver {
                    offset:
                    if chunk_idx > 0 {
                        if let Some(prev_chunk) = self.chunks.get(&(chunk_idx-1)) {
                            let prev_chunk = prev_chunk.read().unwrap();
                            prev_chunk.offset + prev_chunk.view.len().unwrap_or(0)
                        } else {
                            0
                        }
                    } else {
                        0
                    },
                    view: None,
                    sender
                }
            ));

            self.chunks.insert(chunk_idx, chunk_obs.clone());
            chunk_port.add_observer(chunk_obs);

            Some(recv)
        } else {
            self.chunks.remove(&chunk_idx);
            None
        }
    }

    /// recalculate all chunk offsets beginning at start_idx
    /// and update length of flattened sequence
    fn update_offsets(&mut self, start_idx: usize) -> Vec<usize> {
        let top_len = self.top.read().unwrap().view.len().unwrap_or(0);

        let first_chunk = self.chunks.get(&start_idx).unwrap().read().unwrap();
        let mut start_offset = first_chunk.offset + first_chunk.view.len().unwrap_or(0);
        let mut cur_offset = start_offset;

        let mut dirty_idx = Vec::new();

        for chunk_idx in start_idx+1 .. top_len {
            if let Some(cur_chunk) = self.chunks.get(&chunk_idx) {
                let mut cur_chunk = cur_chunk.write().unwrap();

                let chunk_len = cur_chunk.view.len().unwrap_or(0);
                let old_offset = cur_chunk.offset;
                cur_chunk.offset = cur_offset;

                if old_offset != cur_offset {
                    dirty_idx.extend(
                        std::cmp::min(old_offset, cur_offset)
                            .. std::cmp::max(old_offset, cur_offset) + chunk_len
                    );
                }
                cur_offset += chunk_len;
            }
        }

        let old_length = self.length;
        self.length = cur_offset;

        dirty_idx.extend(self.length .. old_length);
        dirty_idx
    }

    /// given an index in the flattened sequence,
    /// which sub-sequence does it belong to?
    fn get_chunk_idx(&self, glob_idx: usize) -> Option<usize> {
        for chunk_idx in 0 .. self.top.read().unwrap().view.len().unwrap_or(0) {
            if let Some(cur_chunk) = self.chunks.get(&chunk_idx) {
                let cur_chunk = cur_chunk.read().unwrap();
                if glob_idx < cur_chunk.offset + cur_chunk.view.len().unwrap_or(0) {
                    return Some(chunk_idx)
                }
            }
        }
        None
    }
}

