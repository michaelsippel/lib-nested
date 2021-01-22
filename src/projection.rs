use {
    std::{
        sync::{Arc, RwLock, Weak},
        cmp::{max}
    },
    crate::{
        core::{View, Observer, ObserverExt},
        singleton::{SingletonView},
        sequence::{SequenceView},
        index::{IndexView}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Special Observer which can access the state of the projection on notify
/// also handles the reset() and default behaviour of unitinitalized inputs
pub struct ProjectionArg<V, P>
where V: View + ?Sized,
      P: Send + Sync {
    pub src: Arc<RwLock<Option<Arc<V>>>>,
    pub proj: Weak<RwLock<P>>,
    notify_fn: Box<dyn Fn(Arc<RwLock<P>>, &V::Msg) + Send + Sync>
}

impl<V, P> ProjectionArg<V, P>
where V: View + ?Sized,
      P: Send + Sync {
    pub fn new(f: impl Fn(Arc<RwLock<P>>, &V::Msg) + Send + Sync + 'static) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(ProjectionArg {
            src: Arc::new(RwLock::new(None)),
            proj: Weak::new(),
            notify_fn: Box::new(f)
        }))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, P> Observer<dyn SingletonView<Item = Item>> for ProjectionArg<dyn SingletonView<Item = Item>, P>
where P: Send + Sync {
    fn reset(&mut self, new_src: Option<Arc<dyn SingletonView<Item = Item>>>) {
        *self.src.write().unwrap() = new_src;
        self.notify(&());
    }

    fn notify(&self, msg: &()) {
        (self.notify_fn)(self.proj.upgrade().unwrap(), msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item, P> Observer<dyn SequenceView<Item = Item>> for ProjectionArg<dyn SequenceView<Item = Item>, P>
where P: Send + Sync {
    fn reset(&mut self, new_src: Option<Arc<dyn SequenceView<Item = Item>>>) {
        let old_len = self.src.len().unwrap_or(0);
        *self.src.write().unwrap() = new_src;
        let new_len = self.src.len().unwrap_or(0);

        self.notify_each(0 .. max(old_len, new_len));
    }

    fn notify(&self, msg: &usize) {
        (self.notify_fn)(self.proj.upgrade().unwrap(), msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key, Item, P> Observer<dyn IndexView<Key, Item = Item>> for ProjectionArg<dyn IndexView<Key, Item = Item>, P>
where P: Send + Sync {
    fn reset(&mut self, new_src: Option<Arc<dyn IndexView<Key, Item = Item>>>) {
        let old_area = self.src.area();
        *self.src.write().unwrap() = new_src;
        let new_area = self.src.area();

        if let Some(area) = old_area { self.notify_each(area); }
        if let Some(area) = new_area { self.notify_each(area); }
    }

    fn notify(&self, msg: &Key) {
        (self.notify_fn)(self.proj.upgrade().unwrap(), msg);
    }
}

