use {
    std::{
        sync::{Arc, RwLock},
        ops::{Deref, Range}
    },
    cgmath::{Point2, Vector2},
    crate::{
        core::View,
        view::{IndexView, ImplIndexView}
    }
};

pub mod offset;
pub use offset::GridOffset;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait GridView = IndexView<Point2<i16>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct GridWindowIterator {
    next: Point2<i16>,
    range: Range<Point2<i16>>
}

impl From<Range<Point2<i16>>> for GridWindowIterator {
    fn from(range: Range<Point2<i16>>) -> Self {
        GridWindowIterator {
            next: range.start,
            range
        }
    }
}

impl Iterator for GridWindowIterator {
    type Item = Point2<i16>;

    fn next(&mut self) -> Option<Point2<i16>> {
        if self.next.y < self.range.end.y {
            let next = self.next;

            if self.next.x+1 < self.range.end.x {
                self.next.x += 1;
            } else {
                self.next.x = self.range.start.x;
                self.next.y += 1;
            }

            Some(next)
        } else {
            None
        }
    }
}

