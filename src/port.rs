use {
    std::{
        sync::{Arc, Weak, RwLock},
        collections::HashSet,
        hash::Hash,
    },
    crate::{
        view::{View, Observer, FnView, FnObserver},
        channel::{ChannelReceiver}
    }
};

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 View Port
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
#[derive(Clone)]
pub struct ViewPort<K: Send + Sync + 'static, V: Send + Sync + 'static> {
    view: Arc<RwLock<Option<Arc<dyn View<Key = K, Value = V>>>>>,
    observers: Arc<RwLock<Vec<Arc<dyn Observer<Msg = K>>>>>
}

impl<K, V> ViewPort<K, V>
where K: Send + Sync + 'static,
      V: Send + Sync + 'static {
    pub fn new() -> Self {
        ViewPort {
            view: Arc::new(RwLock::new(None)),
            observers: Arc::new(RwLock::new(Vec::new()))
        }
    }

    pub fn set_view(&self, view: Arc<dyn View<Key = K, Value = V>>) {
        *self.view.write().unwrap() = Some(view);
    }

    pub fn add_observer(&self, observer: Arc<dyn Observer<Msg = K>>) {
        self.observers.write().unwrap().push(observer);
    }

    pub fn inner(&self) -> InnerViewPort<K, V> {
        InnerViewPort(ViewPort{ view: self.view.clone(), observers: self.observers.clone() })
    }

    pub fn outer(&self) -> OuterViewPort<K, V> {
        OuterViewPort(ViewPort{ view: self.view.clone(), observers: self.observers.clone() })
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct InnerViewPort<K: Send + Sync + 'static, V: Send + Sync + 'static>(ViewPort<K, V>);

#[derive(Clone)]
pub struct OuterViewPort<K: Send + Sync + 'static, V: Send + Sync + 'static>(ViewPort<K, V>);

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<K: Send + Sync + 'static, V: Send + Sync + 'static> OuterViewPort<K, V> {
    pub fn get_view(&self) -> Arc<RwLock<Option<Arc<dyn View<Key = K, Value = V>>>>> {
        self.0.view.clone()
    }

    pub fn add_observer(self, observer: Arc<dyn Observer<Msg = K>>) -> Arc<RwLock<Option<Arc<dyn View<Key = K, Value = V>>>>> {
        self.0.add_observer(observer);
        self.0.view
    }

    pub fn add_observer_fn(self, obs_fn: impl Fn(K) + Send + Sync + 'static) -> Arc<RwLock<Option<Arc<dyn View<Key = K, Value = V>>>>> {
        self.add_observer(Arc::new(FnObserver::new(obs_fn)))
    }
}

impl<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> OuterViewPort<K, V> {
    pub fn stream(&self) -> ChannelReceiver<HashSet<K>> {
        let (s, r) = crate::channel::set_channel();
        self.0.add_observer(Arc::new(s));
        r
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<K: Send + Sync + 'static, V: Send + Sync + 'static> InnerViewPort<K, V> {
    pub fn set_view(&self, view: Arc<dyn View<Key = K, Value = V> + Send + Sync>) {
        *self.0.view.write().unwrap() = Some(view);
    }

    pub fn set_view_fn(&self, view_fn: impl Fn(K) -> Option<V> + Send + Sync + 'static) {
        self.set_view(Arc::new(FnView::new(view_fn)))
    }
}

impl<K, V> Observer for InnerViewPort<K, V>
where K: Clone + Send + Sync + 'static,
      V: Send + Sync + 'static {
    type Msg = K;

    fn notify(&self, msg: K) {
        for observer in self.0.observers.read().unwrap().iter() {
            observer.notify(msg.clone());
        }
    }    
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                Stream Port
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/

/*
pub struct StreamPort<T> {
    actions: Vec<Arc<Mutex<dyn FnMut(T)>>>
}

impl<T> StreamPort<T> {
    async fn set_stream(&self, stream: impl Stream<T>) -> impl Future<()> {
        for msg in stream.next().await.unwrap() {
            for act in self.actions.iter() {
                (*act.lock().unwrap())(msg);
            }
        }
    }

    fn add_action(&self, action: impl FnMut(T)) {
        self.actions.push(Arc::new(Mutex::new(action)))
    }
}
 */


