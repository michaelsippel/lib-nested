
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

    fn get(&self, key: &Key) -> Option<Self::Item>;

    // todo: AreaIterator enum to switch between Allocated and Procedural area
    fn area(&self) -> Option<Vec<Key>> {
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key, V: IndexView<Key> + ?Sized> IndexView<Key> for RwLock<V> {
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.read().unwrap().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.read().unwrap().area()
    }
}

impl<Key, V: IndexView<Key> + ?Sized> IndexView<Key> for Arc<V> {
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.deref().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.deref().area()
    }
}

impl<Key, V: IndexView<Key>> IndexView<Key> for Option<V> {
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        (self.as_ref()? as &V).get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        if let Some(v) = self.as_ref() {
            v.area()
        } else {
            Some(Vec::new())
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ImplIndexView : Send + Sync {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<Self::Value>;
    fn area(&self) -> Option<Vec<Self::Key>> {
        None
    }    
}

impl<V: ImplIndexView> View for V {
    type Msg = V::Key;
}

impl<V: ImplIndexView> IndexView<V::Key> for V {
    type Item = V::Value;

    fn get(&self, key: &V::Key) -> Option<Self::Item> {
        (self as &V).get(key)
    }

    fn area(&self) -> Option<Vec<V::Key>> {
        (self as &V).area()
    }
}

