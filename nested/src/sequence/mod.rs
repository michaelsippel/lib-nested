
pub mod seq2idx;
pub mod map;
pub mod filter;
pub mod flatten;

pub use seq2idx::{Sequence2Index};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use crate::core::View;

pub trait SequenceView : View<Msg = usize> {
    type Item;

    fn get(&self, idx: &usize) -> Option<Self::Item>;
    fn len(&self) -> Option<usize>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use std::{
    sync::Arc,
    ops::{Deref}
};
use std::sync::RwLock;

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

