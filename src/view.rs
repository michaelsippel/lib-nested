
//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait View {
    type Key;
    type Value;

    fn view(&self, key: Self::Key) -> Option<Self::Value>;
}

pub trait Observer {
    type Msg;

    fn notify(&mut self, key: Self::Msg);
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

pub struct FnView<K, V, F: Fn(K) -> Option<V>> {
    f: F,
    _phantom0: std::marker::PhantomData<K>,
    _phantom1: std::marker::PhantomData<V>
}

impl<K, V, F> FnView<K, V, F>
where F: Fn(K) -> Option<V> {
    pub fn new(f: F) -> Self {
        FnView {
            f,
            _phantom0: std::marker::PhantomData,
            _phantom1: std::marker::PhantomData
        }
    }
}

impl<K, V, F> View for FnView<K, V, F>
where F: Fn(K) -> Option<V> {
    type Key = K;
    type Value = V;

    fn view(&self, key: K) -> Option<V> {
        (self.f)(key)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct FnObserver<T, F: FnMut(T)> {
    f: F,
    _phantom: std::marker::PhantomData<T>
}

impl<T, F> FnObserver<T, F>
where F: FnMut(T) {
    pub fn new(f: F) -> Self {
        FnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<T, F> Observer for FnObserver<T, F>
where F: FnMut(T) {
    type Msg = T;

    fn notify(&mut self, key: T) {
        (self.f)(key);
    }
}

