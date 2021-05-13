
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
impl<T> OuterViewPort<dyn SingletonView<Item = T>> {
    pub fn get(&self) -> T {
        self.get_view().unrwap().read().unwrap().get();
    }

    pub fn map<U: Send + Sync + 'static>(&self, f: impl Fn(T) -> U) -> OuterViewPort<dyn SingletonView<Item = U>> {
        
    }
}
 */

