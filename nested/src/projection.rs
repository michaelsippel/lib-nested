use {
    std::{
        cmp::{max},
        any::Any,
        sync::{Arc, Weak},
    },
    async_std::{
        stream::StreamExt
    },
    std::sync::RwLock,
    crate::{
        core::{
            View,
            Observer, ObserverExt,
            OuterViewPort,
            channel::{
                channel,
                ChannelData,
                ChannelSender
            }
        },
        singleton::{SingletonView},
        sequence::{SequenceView},
        index::{IndexView}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProjectionHelper<P: Send + Sync + 'static> {
    keepalive: Vec<Arc<dyn Any + Send + Sync>>,
    proj: Arc<RwLock<Weak<RwLock<P>>>>
}

impl<P: Send + Sync + 'static> ProjectionHelper<P> {
    pub fn new() -> Self {
        ProjectionHelper {
            keepalive: Vec::new(),
            proj: Arc::new(RwLock::new(Weak::new()))
        }
    }

    pub fn set_proj(&mut self, proj: &Arc<RwLock<P>>) {
        *self.proj.write().unwrap() = Arc::downgrade(proj);
    }

    // todo: make this functions generic over the View
    // this does currently not work because Observer<V> is not implemented for ProjectionArg for *all* V.

    pub fn new_singleton_arg<Item: 'static>(
        &mut self,
        port: OuterViewPort<dyn SingletonView<Item = Item>>,
        notify: impl Fn(&mut P, &()) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn SingletonView<Item = Item>>>>> {
        port.add_observer(self.new_arg(notify));
        port.get_view_arc()
    }

    pub fn new_sequence_arg<Item: 'static>(
        &mut self,
        port: OuterViewPort<dyn SequenceView<Item = Item>>,
        notify: impl Fn(&mut P, &usize) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn SequenceView<Item = Item>>>>> {
        port.add_observer(self.new_arg(notify));
        port.get_view_arc()
    }

    pub fn new_index_arg<Key: Clone + Send + Sync + 'static, Item: 'static>(
        &mut self,
        port: OuterViewPort<dyn IndexView<Key, Item = Item>>,
        notify: impl Fn(&mut P, &Key) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn IndexView<Key, Item = Item>>>>> {
        port.add_observer(self.new_arg(notify));
        port.get_view_arc()
    }

    pub fn new_arg<
        V: View + ?Sized + 'static
    >(
        &mut self,
        notify: impl Fn(&mut P, &V::Msg) + Send + Sync + 'static
    ) -> Arc<RwLock<ProjectionArg<V, Vec<V::Msg>>>>
    where V::Msg: Send + Sync {
        let (tx, mut rx) = channel::<Vec<V::Msg>>();

        let arg = Arc::new(RwLock::new(
            ProjectionArg {
                src: None,
                sender: tx
            }));

        let proj = self.proj.clone();
        async_std::task::spawn(async move {
            while let Some(msg) = rx.next().await {
                if let Some(proj) = proj.read().unwrap().upgrade() {
                    notify(&mut *proj.write().unwrap(), &msg);
                }
            }
        });

        self.keepalive.push(arg.clone());

        arg
    }
}

/// Special Observer which can access the state of the projection on notify
/// also handles the reset() and default behaviour of unitinitalized inputs
pub struct ProjectionArg<V, D>
where V: View + ?Sized,
      D: ChannelData<Item = V::Msg>,
      D::IntoIter: Send + Sync
{
    src: Option<Arc<V>>,
    sender: ChannelSender<D>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, D> Observer<dyn SingletonView<Item = Item>> for ProjectionArg<dyn SingletonView<Item = Item>, D>
where D: ChannelData<Item = ()>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn SingletonView<Item = Item>>>) {
        self.src = new_src;
        self.notify(&());
    }

    fn notify(&self, msg: &()) {
        self.sender.send(*msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, D> Observer<dyn SequenceView<Item = Item>> for ProjectionArg<dyn SequenceView<Item = Item>, D>
where D: ChannelData<Item = usize>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn SequenceView<Item = Item>>>) {
        let old_len = self.src.len().unwrap_or(0);
        self.src = new_src;
        let new_len = self.src.len().unwrap_or(0);

        self.notify_each(0 .. max(old_len, new_len));
    }

    fn notify(&self, msg: &usize) {
        self.sender.send(*msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key: Clone, Item, D> Observer<dyn IndexView<Key, Item = Item>> for ProjectionArg<dyn IndexView<Key, Item = Item>, D>
where D: ChannelData<Item = Key>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn IndexView<Key, Item = Item>>>) {
        let old_area = self.src.area();
        self.src = new_src;
        let new_area = self.src.area();

        if let Some(area) = old_area { self.notify_each(area); }
        if let Some(area) = new_area { self.notify_each(area); }
    }

    fn notify(&self, msg: &Key) {
        self.sender.send(msg.clone());
    }
}

