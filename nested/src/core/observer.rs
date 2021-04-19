use {
    crate::core::View,
    std::{
        sync::{Arc, Weak}
    },
    std::sync::RwLock
};

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Observer
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub trait Observer<V: View + ?Sized> : Send + Sync {
    fn reset(&mut self, _view: Option<Arc<V>>) {}
    fn notify(&self, msg: &V::Msg);
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: View + ?Sized, O: Observer<V>> Observer<V> for Arc<RwLock<O>> {
    fn reset(&mut self, view: Option<Arc<V>>) {
        self.write().unwrap().reset(view);
    }

    fn notify(&self, msg: &V::Msg) {
        self.read().unwrap().notify(&msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ObserverExt<V: View + ?Sized> : Observer<V> {    
    fn notify_each(&self, it: impl IntoIterator<Item = V::Msg>);
}

impl<V: View + ?Sized, T: Observer<V>> ObserverExt<V> for T {
    fn notify_each(&self, it: impl IntoIterator<Item = V::Msg>) {
        for msg in it {
            self.notify(&msg);
        }
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Broadcast
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/

pub struct ObserverBroadcast<V: View + ?Sized> {
    observers: Vec<Weak<RwLock<dyn Observer<V>>>>
}

impl<V: View + ?Sized> ObserverBroadcast<V> {
    pub fn new() -> Self {
        ObserverBroadcast {
            observers: Vec::new()
        }
    }

    pub fn add_observer(&mut self, obs: Weak<RwLock<dyn Observer<V>>>) {
        self.cleanup();
        self.observers.push(obs);
    }

    fn cleanup(&mut self) {
        self.observers.retain(|o| o.strong_count() > 0);
    }

    fn iter(&self) -> impl Iterator<Item = Arc<RwLock<dyn Observer<V>>>> + '_ {
        self.observers.iter().filter_map(|o| o.upgrade())
    }
}

impl<V: View + ?Sized> Observer<V> for ObserverBroadcast<V> {
    fn reset(&mut self, view: Option<Arc<V>>) {
        for o in self.iter() {
            o.write().unwrap().reset(view.clone());
        }
    }

    fn notify(&self, msg: &V::Msg) {
        for o in self.iter() {
            o.read().unwrap().notify(&msg);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    f: F,
    _phantom: std::marker::PhantomData<V>
}

impl<V, F> NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    pub fn new(f: F) -> Self {
        NotifyFnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<V, F> Observer<V> for NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    fn notify(&self, msg: &V::Msg) {
        (self.f)(msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    f: F,
    _phantom: std::marker::PhantomData<V>
}

impl<V, F> ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    pub fn new(f: F) -> Self {
        ResetFnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<V, F> Observer<V> for ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    fn notify(&self, _msg: &V::Msg) {}
    fn reset(&mut self, view: Option<Arc<V>>) {
        (self.f)(view);
    }
}


