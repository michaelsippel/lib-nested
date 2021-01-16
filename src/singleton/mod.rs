
pub mod buffer;

use {
    std::{
        sync::{Arc, RwLock},
        ops::Deref
    },
    crate::core::{View}
};

pub use buffer::SingletonBuffer;

// TODO: #[ImplForArc, ImplForRwLock]
pub trait SingletonView : View<Msg = ()> {
    type Item;

    fn get(&self) -> Self::Item;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: SingletonView> SingletonView for RwLock<V> {
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        self.read().unwrap().get()
    }
}

impl<V: SingletonView> SingletonView for Arc<V> {
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        self.deref().get()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
/*
pub trait ImplSingletonView : Send + Sync {
    type Item;

    fn get(&self) -> Self::Item;
}

impl<V: ImplSingletonView> View for V {
    type Msg = ();
}

impl<V: ImplSingletonView> SingletonView for V {
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        (self as &V).get()
    }
}
*/
