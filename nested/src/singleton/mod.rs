
pub mod buffer;

use {
    std::{
        sync::Arc,
        ops::Deref
    },
    std::sync::RwLock,
    crate::core::{View}
};

pub use buffer::SingletonBuffer;

// TODO: #[ImplForArc, ImplForRwLock]
pub trait SingletonView : View<Msg = ()> {
    type Item;

    fn get(&self) -> Self::Item;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: SingletonView + ?Sized> SingletonView for RwLock<V> {
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        self.read().unwrap().get()
    }
}

impl<V: SingletonView + ?Sized> SingletonView for Arc<V> {
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        self.deref().get()
    }
}

impl<V: SingletonView> SingletonView for Option<V>
where V::Item: Default{
    type Item = V::Item;

    fn get(&self) -> Self::Item {
        if let Some(s) = self.as_ref() {
            s.get()
        } else {
            V::Item::default()
        }
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
