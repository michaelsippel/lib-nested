use {
    std::{
        cmp::{max},
        any::Any,
        sync::{Arc, Weak},
        hash::Hash,
        collections::HashMap
    },
    std::sync::RwLock,
    crate::{
        core::{
            View,
            Observer, ObserverExt,
            port::UpdateTask,
            OuterViewPort,
            channel::{
                ChannelSender, ChannelReceiver,
                ChannelData,
                set_channel
            }
        },
        singleton::{SingletonView},
        sequence::{SequenceView},
        index::{IndexView}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProjectionHelper<ArgKey, P>
where ArgKey: Clone + Hash + Eq,
      P: Send + Sync + 'static
{
    keepalive: HashMap<ArgKey, (usize, Arc<dyn Any + Send + Sync>)>,
    proj: Arc<RwLock<Weak<RwLock<P>>>>,
    update_hooks: Arc<RwLock<Vec<Arc<dyn UpdateTask>>>>
}

impl<ArgKey, P> ProjectionHelper<ArgKey, P>
where ArgKey: Clone + Hash + Eq,
      P: Send + Sync + 'static
{
    pub fn new(update_hooks: Arc<RwLock<Vec<Arc<dyn UpdateTask>>>>) -> Self {
        ProjectionHelper {
            keepalive: HashMap::new(),
            proj: Arc::new(RwLock::new(Weak::new())),
            update_hooks
        }
    }

    pub fn set_proj(&mut self, proj: &Arc<RwLock<P>>) {
        *self.proj.write().unwrap() = Arc::downgrade(proj);
    }

    // todo: make this functions generic over the View
    // this does currently not work because Observer<V> is not implemented for ProjectionArg for *all* V.

    pub fn new_singleton_arg<Item: 'static>(
        &mut self,
        arg_key: ArgKey,
        port: OuterViewPort<dyn SingletonView<Item = Item>>,
        notify: impl Fn(&mut P, &()) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn SingletonView<Item = Item>>>>> {
        port.add_observer(self.new_arg(arg_key, Arc::new(port.0.clone()), notify, set_channel()));
        port.get_view_arc()
    }

    pub fn new_sequence_arg<Item: 'static>(
        &mut self,
        arg_key: ArgKey,
        port: OuterViewPort<dyn SequenceView<Item = Item>>,
        notify: impl Fn(&mut P, &usize) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn SequenceView<Item = Item>>>>> {
        port.add_observer(self.new_arg(arg_key, Arc::new(port.0.clone()), notify, set_channel()));
        port.get_view_arc()
    }

    pub fn new_index_arg<Key: Hash + Eq + Clone + Send + Sync + std::fmt::Debug + 'static, Item: 'static>(
        &mut self,
        arg_key: ArgKey,
        port: OuterViewPort<dyn IndexView<Key, Item = Item>>,
        notify: impl Fn(&mut P, &Key) + Send + Sync + 'static
    ) -> Arc<RwLock<Option<Arc<dyn IndexView<Key, Item = Item>>>>> {
        port.add_observer(self.new_arg(arg_key, Arc::new(port.0.clone()), notify, set_channel()));
        port.get_view_arc()
    }

    pub fn new_arg<
        V: View + ?Sized + 'static,
        D: ChannelData<Item = V::Msg> + 'static
    >(
        &mut self,
        arg_key: ArgKey,
        src_update: Arc<dyn UpdateTask>,
        notify: impl Fn(&mut P, &V::Msg) + Send + Sync + 'static,
        (tx, rx): (ChannelSender<D>, ChannelReceiver<D>)
    )
        -> Arc<RwLock<ProjectionArg<P, V, D>>>
    where V::Msg: Send + Sync + std::fmt::Debug,
          D::IntoIter: Send + Sync + 'static
    {
        self.remove_arg(&arg_key);

        let arg = Arc::new(RwLock::new(
            ProjectionArg {
                src: None,
                notify: Box::new(notify),
                proj: self.proj.clone(),
                rx, tx
            }));

        let mut hooks = self.update_hooks.write().unwrap();
        let idx = hooks.len();
        hooks.push(src_update);
        hooks.push(arg.clone());
        self.keepalive.insert(arg_key, (idx, arg.clone()));

        arg
    }

    pub fn remove_arg(&mut self, arg_key: &ArgKey) {
        let mut hooks = self.update_hooks.write().unwrap();
        if let Some((idx, arg)) = self.keepalive.remove(arg_key) {
            hooks.remove(idx);
            hooks.remove(idx);
            for (_, (j, _)) in self.keepalive.iter_mut() {
                if *j > idx {
                    *j -= 2;
                }
            }
        }
    }
}

/// Special Observer which can access the state of the projection on notify
/// also handles the reset()
pub struct ProjectionArg<P, V, D>
where P: Send + Sync + 'static,
      V: View + ?Sized,
      D: ChannelData<Item = V::Msg>,
      D::IntoIter: Send + Sync
{
    src: Option<Arc<V>>,
    notify: Box<dyn Fn(&mut P, &V::Msg) + Send + Sync + 'static>,
    proj: Arc<RwLock<Weak<RwLock<P>>>>,
    rx: ChannelReceiver<D>,
    tx: ChannelSender<D>
}

impl<P, V, D> UpdateTask for ProjectionArg<P, V, D>
where P: Send + Sync + 'static,
      V: View + ?Sized,
      D: ChannelData<Item = V::Msg>,
      D::IntoIter: Send + Sync,
V::Msg: std::fmt::Debug
{
    fn update(&self) {
        if let Some(p) = self.proj.read().unwrap().upgrade() {
            if let Some(data) = self.rx.try_recv() {
                for msg in data {
                    //eprintln!("proj update {:?}", msg);
                    (self.notify)(
                        &mut *p.write().unwrap(),
                        &msg
                    );
                }
            }
        } else {
            eprintln!("proj update: upgrade fail");
        }
    }
}

impl<P, V, D> UpdateTask for RwLock<ProjectionArg<P, V, D>>
where P: Send + Sync + 'static,
      V: View + ?Sized,
      D: ChannelData<Item = V::Msg>,
      D::IntoIter: Send + Sync,
V::Msg: std::fmt::Debug
{
    fn update(&self) {
        self.read().unwrap().update();
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<P, Item, D> Observer<dyn SingletonView<Item = Item>> for ProjectionArg<P, dyn SingletonView<Item = Item>, D>
where P: Send + Sync + 'static,
      D: ChannelData<Item = ()>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn SingletonView<Item = Item>>>) {
        self.src = new_src;
        self.notify(&());
    }

    fn notify(&mut self, msg: &()) {
        self.tx.send(msg.clone());
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<P, Item, D> Observer<dyn SequenceView<Item = Item>> for ProjectionArg<P, dyn SequenceView<Item = Item>, D>
where P: Send + Sync + 'static,
      D: ChannelData<Item = usize>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn SequenceView<Item = Item>>>) {
        let old_len = self.src.len().unwrap_or(0);
        self.src = new_src;
        let new_len = self.src.len().unwrap_or(0);

        self.notify_each(0 .. max(old_len, new_len));
    }

    fn notify(&mut self, msg: &usize) {
        self.tx.send(*msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<P, Key, Item, D> Observer<dyn IndexView<Key, Item = Item>> for ProjectionArg<P, dyn IndexView<Key, Item = Item>, D>
where P: Send + Sync + 'static,
      Key: Clone + Send + Sync,
      D: ChannelData<Item = Key>,
      D::IntoIter: Send + Sync
{
    fn reset(&mut self, new_src: Option<Arc<dyn IndexView<Key, Item = Item>>>) {
        let old_area = self.src.area();
        self.src = new_src;
        let new_area = self.src.area();

        if let Some(area) = old_area { self.notify_each(area); }
        if let Some(area) = new_area { self.notify_each(area); }
    }

    fn notify(&mut self, msg: &Key) {
        self.tx.send(msg.clone());
    }
}

