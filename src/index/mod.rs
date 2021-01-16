
pub mod map_item;
pub mod map_key;

use {
    std::{
        sync::{Arc, RwLock},
        ops::Deref,
    },
    crate::core::View
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait IndexView<Key> : View<Msg = Key> {
    type Item;

    fn get(&self, key: &Key) -> Self::Item;

    fn area(&self) -> Option<Vec<Key>> {
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key, V: IndexView<Key>> IndexView<Key> for RwLock<V> {
    type Item = V::Item;

    fn get(&self, key: &Key) -> Self::Item {
        self.read().unwrap().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.read().unwrap().area()
    }
}

impl<Key, V: IndexView<Key>> IndexView<Key> for Arc<V> {
    type Item = V::Item;

    fn get(&self, key: &Key) -> Self::Item {
        self.deref().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.deref().area()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ImplIndexView : Send + Sync {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Self::Value;
    fn area(&self) -> Option<Vec<Self::Key>> {
        None
    }    
}

impl<V: ImplIndexView> View for V {
    type Msg = V::Key;
}

impl<V: ImplIndexView> IndexView<V::Key> for V {
    type Item = V::Value;

    fn get(&self, key: &V::Key) -> Self::Item {
        (self as &V).get(key)
    }

    fn area(&self) -> Option<Vec<V::Key>> {
        (self as &V).area()
    }
}

