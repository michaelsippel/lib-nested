
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait View : Send + Sync {
    type Key;
    type Value;

    fn view(&self, key: Self::Key) -> Option<Self::Value>;
}

pub trait Observer : Send + Sync {
    type Msg;

    fn notify(&self, key: Self::Msg);
}

pub trait ObserverExt : Observer {
    fn notify_each(&self, it: impl IntoIterator<Item = Self::Msg>);
}

impl<T: Observer> ObserverExt for T {
    fn notify_each(&self, it: impl IntoIterator<Item = Self::Msg>) {
        for msg in it {
            self.notify(msg);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use cgmath::Vector2;

pub trait SingletonView = View<Key = ()>;
pub trait SingletonObserver = Observer<Msg = ()>;

pub trait SequenceView = View<Key = usize>;
pub trait SequenceObserver = Observer<Msg = usize>;

pub trait GridView = View<Key = Vector2<i16>>;
pub trait GridObserver = Observer<Msg = Vector2<i16>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct FnView<K, V, F>
where K: Send + Sync,
      V: Send + Sync,
      F: Fn(K) -> Option<V> + Send + Sync {
    f: F,
    _phantom0: std::marker::PhantomData<K>,
    _phantom1: std::marker::PhantomData<V>
}

impl<K, V, F> FnView<K, V, F>
where K: Send + Sync,
      V: Send + Sync,
      F: Fn(K) -> Option<V> + Send + Sync {
    pub fn new(f: F) -> Self {
        FnView {
            f,
            _phantom0: std::marker::PhantomData,
            _phantom1: std::marker::PhantomData
        }
    }
}

impl<K, V, F> View for FnView<K, V, F>
where K: Send + Sync,
      V: Send + Sync,
      F: Fn(K) -> Option<V> + Send + Sync {
    type Key = K;
    type Value = V;

    fn view(&self, key: K) -> Option<V> {
        (self.f)(key)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct FnObserver<T, F>
where T: Send + Sync,
      F: Fn(T) + Send + Sync {
    f: F,
    _phantom: std::marker::PhantomData<T>
}

impl<T, F> FnObserver<T, F>
where T: Send + Sync,
      F: Fn(T) + Send + Sync {
    pub fn new(f: F) -> Self {
        FnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<T, F> Observer for FnObserver<T, F>
where T: Send + Sync,
      F: Fn(T) + Send + Sync {
    type Msg = T;

    fn notify(&self, msg: T) {
        (self.f)(msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::ops::Deref;
use std::sync::{Arc, RwLock};

impl<T: View> View for RwLock<T> {
    type Key = T::Key;
    type Value = T::Value;

    fn view(&self, key: T::Key) -> Option<T::Value> {
        self.read().unwrap().view(key)
    }
}

impl<T: Observer> Observer for RwLock<T> {
    type Msg = T::Msg;

    fn notify(&self, msg: T::Msg) {
        self.read().unwrap().notify(msg)
    }
}

impl<T: View> View for Arc<T> {
    type Key = T::Key;
    type Value = T::Value;

    fn view(&self, key: T::Key) -> Option<T::Value> {
        self.deref().view(key)
    }
}

impl<T: Observer> Observer for Arc<T> {
    type Msg = T::Msg;

    fn notify(&self, msg: T::Msg) {
        self.deref().notify(msg)
    }
}

impl<K, V> View for Arc<dyn View<Key = K, Value = V>>
where K: Send + Sync,
      V: Send + Sync {
    type Key = K;
    type Value = V;

    fn view(&self, key: K) -> Option<V> {
        self.deref().view(key)
    }
}

impl<T> Observer for Arc<dyn Observer<Msg = T>>
where T:  Send + Sync {
    type Msg = T;

    fn notify(&self, msg: T) {
        self.deref().notify(msg)
    }
}

impl<T: View> View for Option<T> {
    type Key = T::Key;
    type Value = T::Value;

    fn view(&self, key: T::Key) -> Option<T::Value> {
        if let Some(view) = self.as_ref() {
            view.view(key)
        } else {
            None
        }
    }
}

impl<T: Observer> Observer for Option<T> {
    type Msg = T::Msg;

    fn notify(&self, msg: T::Msg) {
        if let Some(obs) = self.as_ref() {
            obs.notify(msg);
        }
    }
}

