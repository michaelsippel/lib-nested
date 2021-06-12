use {
    std::{
        ops::{Range, RangeInclusive}
    },
    cgmath::{Point2},
    crate::{
        index::{IndexView}
    }
};

pub mod offset;
pub mod flatten;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait GridView = IndexView<Point2<i16>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item> dyn GridView<Item = Item> {
    pub fn range(&self) -> Range<Point2<i16>> {
        if let Some(area) = self.area() {
            Point2::new(
                area.iter().map(|p| p.x).min().unwrap_or(0),
                area.iter().map(|p| p.y).min().unwrap_or(0)
            ) ..
                Point2::new(
                    area.iter().map(|p| p.x+1).max().unwrap_or(0),
                    area.iter().map(|p| p.y+1).max().unwrap_or(0)
                )
        } else {
            Point2::new(i16::MIN, i16::MIN) .. Point2::new(i16::MAX, i16::MAX)
        }
    }
}

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

impl From<RangeInclusive<Point2<i16>>> for GridWindowIterator {
    fn from(range: RangeInclusive<Point2<i16>>) -> Self {
        GridWindowIterator {
            next: *range.start(),
            range: *range.start() .. Point2::new(range.end().x+1, range.end().y+1)
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

