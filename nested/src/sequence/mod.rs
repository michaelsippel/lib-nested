pub mod filter;
pub mod flatten;
pub mod map;
pub mod seq2idx;

pub use seq2idx::Sequence2Index;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use crate::core::View;

pub trait SequenceView: View<Msg = usize> {
    type Item;

    fn get(&self, idx: &usize) -> Option<Self::Item>;
    fn len(&self) -> Option<usize>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait SequenceViewExt: SequenceView {
    fn iter<'a>(&'a self) -> SequenceViewIter<'a, Self> {
        SequenceViewIter { view: self, cur: 0 }
    }
}

impl<V: SequenceView + ?Sized> SequenceViewExt for V {}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct SequenceViewIter<'a, V>
where
    V: SequenceView + ?Sized,
{
    view: &'a V,
    cur: usize,
}

impl<'a, V> Iterator for SequenceViewIter<'a, V>
where
    V: SequenceView + ?Sized,
{
    type Item = V::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.cur;
        self.cur += 1;
        self.view.get(&i)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::sync::RwLock;
use std::{ops::Deref, sync::Arc};

impl<V: SequenceView + ?Sized> SequenceView for RwLock<V> {
    type Item = V::Item;

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        self.read().unwrap().get(idx)
    }

    fn len(&self) -> Option<usize> {
        self.read().unwrap().len()
    }
}

impl<V: SequenceView + ?Sized> SequenceView for Arc<V> {
    type Item = V::Item;

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        self.deref().get(idx)
    }

    fn len(&self) -> Option<usize> {
        self.deref().len()
    }
}

impl<V: SequenceView> SequenceView for Option<V> {
    type Item = V::Item;

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        (self.as_ref()? as &V).get(idx)
    }

    fn len(&self) -> Option<usize> {
        if let Some(v) = self.as_ref() {
            v.len()
        } else {
            Some(0)
        }
    }
}
