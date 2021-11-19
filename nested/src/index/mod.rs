pub mod buffer;
pub mod map_item;
pub mod map_key;

use {
    crate::core::View,
    std::sync::RwLock,
    std::{
        ops::{Deref, RangeInclusive},
        sync::Arc,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub enum IndexArea<Key> {
    Empty,
    Full,
    Set(Vec<Key>),
    Range(RangeInclusive<Key>),
    //Procedural(Arc<dyn Fn() -> Box<dyn Iterator<Item = Key>>>)
}

impl<Key> IndexArea<Key> {
    pub fn map<T>(&self, f: impl Fn(&Key) -> T) -> IndexArea<T> {
        match self {
            IndexArea::Empty => IndexArea::Empty,
            IndexArea::Full => IndexArea::Full,
            IndexArea::Set(v) => IndexArea::Set(v.iter().map(&f).collect()),
            IndexArea::Range(r) => IndexArea::Range(f(&r.start())..=f(&r.end())),
        }
    }
}

pub trait IndexView<Key>: View<Msg = IndexArea<Key>>
where
    Key: Send + Sync,
{
    type Item;

    fn get(&self, key: &Key) -> Option<Self::Item>;

    fn area(&self) -> IndexArea<Key> {
        IndexArea::Full
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Key, V> IndexView<Key> for RwLock<V>
where
    Key: Send + Sync,
    V: IndexView<Key> + ?Sized,
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.read().unwrap().get(key)
    }

    fn area(&self) -> IndexArea<Key> {
        self.read().unwrap().area()
    }
}

impl<Key, V> IndexView<Key> for Arc<V>
where
    Key: Send + Sync,
    V: IndexView<Key> + ?Sized,
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.deref().get(key)
    }

    fn area(&self) -> IndexArea<Key> {
        self.deref().area()
    }
}

impl<Key, V> IndexView<Key> for Option<V>
where
    Key: Send + Sync,
    V: IndexView<Key>,
{
    type Item = V::Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.as_ref()?.get(key)
    }

    fn area(&self) -> IndexArea<Key> {
        if let Some(v) = self.as_ref() {
            v.area()
        } else {
            IndexArea::Empty
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
/*
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
*/
