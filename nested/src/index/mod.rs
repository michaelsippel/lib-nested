
pub mod map_item;
pub mod map_key;
pub mod buffer;

use {
    std::{
        sync::Arc,
        ops::Deref,
    },
    std::sync::RwLock,
    crate::core::View
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait IndexView<Key> : View<Msg = Key>
where Key: Send + Sync {
    type Item;

    fn get(&self, key: &Key) -> Option<Self::Item>;

    // todo: AreaIterator enum to switch between Allocated and Procedural area
    fn area(&self) -> Option<Vec<Key>> {
        None
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key, V> IndexView<Key> for RwLock<V>
where Key: Send + Sync,
      V: IndexView<Key> + ?Sized
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.read().unwrap().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.read().unwrap().area()
    }
}

impl<Key, V> IndexView<Key> for Arc<V>
where Key: Send + Sync,
      V: IndexView<Key> + ?Sized
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.deref().get(key)
    }

    fn area(&self) -> Option<Vec<Key>> {
        self.deref().area()
    }
}

impl<Key, V> IndexView<Key> for Option<V>
where Key: Send + Sync,
      V: IndexView<Key>
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.as_ref()?.get(key)
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
    type Key : Send + Sync;
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

