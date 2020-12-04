
use std::{
    sync::{Arc, Weak, RwLock},
    collections::HashSet,
    hash::Hash
};
use crate::{
    view::{View, Observer, FnView, FnObserver},
    channel::{ChannelReceiver}
};

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 View Port
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/

pub struct ViewPort<K, V> {
    view: Option<Arc<dyn View<Key = K, Value = V> + Send + Sync>>,
    observers: Vec<Arc<RwLock<dyn Observer<Msg = K> + Send + Sync>>>
}

pub fn view_port<K, V>() -> (ViewPortIn<K, V>, ViewPortOut<K, V>) {
    let state = Arc::new(RwLock::new(ViewPort{ view: None, observers: Vec::new() }));
    (ViewPortIn(state.clone()), ViewPortOut(state))
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct ViewPortIn<K, V>(Arc<RwLock<ViewPort<K, V>>>);
impl<K: Send + Sync + 'static, V> ViewPortIn<K, V> {
    pub fn add_observer(&self, observer: Arc<RwLock<dyn Observer<Msg = K> + Send + Sync>>) {
        self.0
            .write().unwrap()
            .observers
            .push(observer);
    }

    pub fn add_observer_fn(&self, obs_fn: impl FnMut(K) + Send + Sync + 'static) {
        self.add_observer(Arc::new(RwLock::new(FnObserver::new(obs_fn))));
    }
}

impl<K: Eq + Hash + Send + Sync + 'static, V> ViewPortIn<K, V> {
    pub fn stream(&self) -> ChannelReceiver<HashSet<K>> {
        let (s, r) = crate::channel::set_channel();
        self.add_observer(Arc::new(RwLock::new(s)));
        r
    }
}

impl<K, V> View for ViewPortIn<K, V> {
    type Key = K;
    type Value = V;

    fn view(&self, key: K) -> Option<V> {
        if let Some(view) = self.0.read().unwrap().view.as_ref() {
            view.view(key)
        } else {
            println!("Warning: trying to access InPort with uninitialized View!");
            None
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ViewPortOut<K, V>(Arc<RwLock<ViewPort<K, V>>>);
impl<K: Send + Sync + 'static, V: Send + Sync + 'static> ViewPortOut<K, V> {
    pub fn set_view(&self, view: Arc<dyn View<Key = K, Value = V> + Send + Sync>) {
        self.0.write().unwrap().view = Some(view);
    }

    pub fn set_view_fn(&self, view_fn: impl Fn(K) -> Option<V> + Send + Sync + 'static) {
        self.set_view(Arc::new(FnView::new(view_fn)))
    }
}

impl<K, V> Observer for ViewPortOut<K, V>
where K: Clone {
    type Msg = K;

    fn notify(&mut self, msg: K) {
        for observer in self.0.read().unwrap().observers.iter() {
            observer.write().unwrap().notify(msg.clone());
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


