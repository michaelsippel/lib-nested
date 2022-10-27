use {
    crate::{
        core::{
            port::UpdateTask, InnerViewPort, Observer, ObserverBroadcast, ObserverExt,
            OuterViewPort, View, ViewPort,
        },
        projection::ProjectionHelper,
        sequence::SequenceView,
    },
    std::sync::RwLock,
    std::{collections::BTreeMap, sync::Arc},
};

impl<Item> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = Item>>>>
where
    Item: 'static,
{
    pub fn flatten(&self) -> OuterViewPort<dyn SequenceView<Item = Item>> {
        let port = ViewPort::new();
        Flatten::new(self.clone(), port.inner());
        port.into_outer()
    }
}

pub struct Chunk<Item>
where
    Item: 'static,
{
    offset: usize,
    len: usize,
    view: Arc<dyn SequenceView<Item = Item>>,
}

pub struct Flatten<Item>
where
    Item: 'static,
{
    length: usize,
    top: Arc<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = Item>>>>,
    chunks: BTreeMap<usize, Chunk<Item>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = Item>>>>,
    proj_helper: ProjectionHelper<usize, Self>,
}

impl<Item> View for Flatten<Item>
where
    Item: 'static,
{
    type Msg = usize;
}

impl<Item> SequenceView for Flatten<Item>
where
    Item: 'static,
{
    type Item = Item;

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let chunk = self.chunks.get(&self.get_chunk_idx(*idx)?)?;
        chunk.view.get(&(*idx - chunk.offset))
    }

    fn len(&self) -> Option<usize> {
        Some(self.length)
    }
}

impl<Item> Flatten<Item>
where
    Item: 'static,
{
    pub fn new(
        top_port: OuterViewPort<
            dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = Item>>>,
        >,
        out_port: InnerViewPort<dyn SequenceView<Item = Item>>,
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let flat = Arc::new(RwLock::new(Flatten {
            length: 0,
            top: proj_helper.new_sequence_arg(usize::MAX, top_port, |s: &mut Self, chunk_idx| {
                s.update_chunk(*chunk_idx);
            }),
            chunks: BTreeMap::new(),
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        flat.write().unwrap().proj_helper.set_proj(&flat);
        out_port.set_view(Some(flat.clone()));
        flat
    }

    /// the top-sequence has changed the item at chunk_idx,
    /// create a new observer for the contained sub sequence
    fn update_chunk(&mut self, chunk_idx: usize) {
        if let Some(chunk_port) = self.top.get(&chunk_idx) {
            self.chunks.insert(
                chunk_idx,
                Chunk {
                    offset: 0, // will be adjusted by update_offsets() later
                    len: 0,
                    view: self.proj_helper.new_sequence_arg(
                        chunk_idx,
                        chunk_port.clone(),
                        move |s: &mut Self, idx| {
                            if let Some(chunk) = s.chunks.get(&chunk_idx) {
                                let chunk_offset = chunk.offset;
                                let chunk_len = chunk.view.len().unwrap_or(0);

                                let mut dirty_idx = Vec::new();
                                if chunk.len != chunk_len {
                                    dirty_idx = s.update_all_offsets();
                                }

                                s.cast.notify(&(idx + chunk_offset));
                                s.cast.notify_each(dirty_idx);
                            } else {
                                let dirty_idx = s.update_all_offsets();
                                s.cast.notify_each(dirty_idx);
                            }
                        },
                    ),
                },
            );

            chunk_port.0.update();
            let dirty_idx = self.update_all_offsets();
            self.cast.notify_each(dirty_idx);
        } else {
            // todo:
            self.proj_helper.remove_arg(&chunk_idx);

            self.chunks.remove(&chunk_idx);

            let dirty_idx = self.update_all_offsets();
            self.cast.notify_each(dirty_idx);
        }
    }

    /// recalculate all chunk offsets beginning at start_idx
    /// and update length of flattened sequence
    fn update_all_offsets(&mut self) -> Vec<usize> {
        let mut dirty_idx = Vec::new();
        let mut cur_offset = 0;

        for (_chunk_idx, chunk) in self.chunks.iter_mut() {
            let old_offset = chunk.offset;
            chunk.offset = cur_offset;
            chunk.len = chunk.view.len().unwrap_or(0);

            if old_offset != cur_offset {
                dirty_idx.extend(
                    std::cmp::min(old_offset, cur_offset)
                        ..std::cmp::max(old_offset, cur_offset) + chunk.len,
                );
            }

            cur_offset += chunk.len;
        }

        let old_length = self.length;
        self.length = cur_offset;

        dirty_idx.extend(self.length..old_length);

        dirty_idx
    }

    /// given an index in the flattened sequence,
    /// which sub-sequence does it belong to?
    fn get_chunk_idx(&self, glob_idx: usize) -> Option<usize> {
        let mut offset = 0;
        for (chunk_idx, chunk) in self.chunks.iter() {
            offset += chunk.view.len().unwrap_or(0);
            if glob_idx < offset {
                return Some(*chunk_idx);
            }
        }
        None
    }
}
