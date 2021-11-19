use {
    crate::index::{IndexArea, IndexView},
    cgmath::Point2,
    std::{
        cmp::{max, min},
        ops::RangeInclusive,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait GridView = IndexView<Point2<i16>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod flatten;
pub mod offset;
pub mod window_iterator;

pub use window_iterator::GridWindowIterator;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl IndexArea<Point2<i16>> {
    // todo: this is not perfect (e.g. diagonals are inefficient)
    pub fn iter(&self) -> GridWindowIterator {
        GridWindowIterator::from(self.range())
    }

    pub fn range(&self) -> RangeInclusive<Point2<i16>> {
        match self {
            IndexArea::Empty => Point2::new(i16::MAX, i16::MAX)..=Point2::new(i16::MIN, i16::MIN),
            IndexArea::Full => panic!("range from full grid area"),
            IndexArea::Set(v) => {
                Point2::new(
                    v.iter().map(|p| p.x).min().unwrap_or(0),
                    v.iter().map(|p| p.y).min().unwrap_or(0),
                )
                    ..=Point2::new(
                        v.iter().map(|p| p.x).max().unwrap_or(0),
                        v.iter().map(|p| p.y).max().unwrap_or(0),
                    )
            }
            IndexArea::Range(r) => r.clone(),
        }
    }

    pub fn union(self, other: IndexArea<Point2<i16>>) -> IndexArea<Point2<i16>> {
        match (self, other) {
            (IndexArea::Empty, a) | (a, IndexArea::Empty) => a,

            (IndexArea::Full, _) | (_, IndexArea::Full) => IndexArea::Full,

            (IndexArea::Set(mut va), IndexArea::Set(vb)) => {
                va.extend(vb.into_iter());
                IndexArea::Set(va)
            }

            (IndexArea::Range(r), IndexArea::Set(mut v))
            | (IndexArea::Set(mut v), IndexArea::Range(r)) => {
                v.extend(GridWindowIterator::from(r));
                IndexArea::Set(v)
            }

            (IndexArea::Range(ra), IndexArea::Range(rb)) => IndexArea::Range(
                Point2::new(
                    min(ra.start().x, rb.start().x),
                    min(ra.start().y, rb.start().y),
                )
                    ..=Point2::new(max(ra.end().x, rb.end().x), max(ra.end().y, rb.end().y)),
            ),
        }
    }
}
