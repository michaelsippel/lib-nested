use {
    std::{
        sync::{Arc, RwLock, Weak},
        cmp::{max},
        any::Any
    },
    async_std::stream::StreamExt,
    crate::{
        core::{View, Observer, ObserverExt, channel::{channel, ChannelData, ChannelSender, ChannelReceiver}},
        singleton::{SingletonView},
        sequence::{SequenceView},
        index::{IndexView}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProjectionHelper<P: Send + Sync + 'static> {
    keepalive: Vec<Arc<dyn Any + Send + Sync>>,
    proj: Weak<RwLock<P>>
}

impl<P: Send + Sync + 'static> ProjectionHelper<P> {
    pub fn new(proj: Weak<RwLock<P>>) -> Self {
        ProjectionHelper {
            keepalive: Vec::new(),
            proj
        }
    }

    pub fn new_arg<
        V: View + ?Sized + 'static
    >(
        &mut self,
        notify: impl Fn(Arc<RwLock<P>>, &V::Msg) + Send + Sync + 'static
    ) -> (
        Arc<RwLock<Option<Arc<V>>>>,
        Arc<RwLock<ProjectionArg<V, Vec<V::Msg>>>>
    ) where V::Msg: Send + Sync {
        let (tx, mut rx) = channel::<Vec<V::Msg>>();

        let view = Arc::new(RwLock::new(None));
        let arg = Arc::new(RwLock::new(
            ProjectionArg {
                src: view.clone(),
                sender: tx
            }));

        let proj = self.proj.clone();
        async_std::task::spawn(async move {
            while let Some(msg) = rx.next().await {
                let proj = proj.upgrade().unwrap();
                notify(proj, &msg);
            }
        });

        self.keepalive.push(arg.clone());
        
        (view, arg)
    }
}

/// Special Observer which can access the state of the projection on notify
/// also handles the reset() and default behaviour of unitinitalized inputs
pub struct ProjectionArg<V, D>
where V: View + ?Sized,
      D: ChannelData<Item = V::Msg>,
      D::IntoIter: Send + Sync
{
    src: Arc<RwLock<Option<Arc<V>>>>,
    sender: ChannelSender<D>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, D> Observer<dyn SingletonView<Item = Item>> for ProjectionArg<dyn SingletonView<Item = Item>, D>
where D: ChannelData<Item = ()>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn SingletonView<Item = Item>>>) {
        *self.src.write().unwrap() = new_src;
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
        *self.src.write().unwrap() = new_src;
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
        *self.src.write().unwrap() = new_src;
        let new_area = self.src.area();

        if let Some(area) = old_area { self.notify_each(area); }
        if let Some(area) = new_area { self.notify_each(area); }
    }

    fn notify(&self, msg: &Key) {
        self.sender.send(msg.clone());
    }
}

