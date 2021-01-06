use {
    std::{
        sync::{Arc, RwLock},
        ops::{Range, Deref}
    },
    super::{IndexView, ImplIndexView},
    crate::core::View
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait SequenceView = IndexView<usize>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
/*
pub trait ImplSequenceView : Send + Sync {
    type Item;

    fn get(&self, idx: usize) -> Self::Item;
    fn len(&self) -> Option<usize> {
        None
    }
}

impl<V: ImplSequenceView> ImplIndexView for V {
    type Key = usize;
    type Value = V::Item;

    fn get(&self, idx: &usize) -> V::Item {
        (self as V).get(*idx)
    }

    fn range(&self) -> Option<Range<usize>> {
        if let Some(len) = (self as V).len() {
            Some(0 .. len)
        } else {
            None
        }
    }
}
*/
