
use {
    std::ops::{Range, RangeInclusive},
    cgmath::{Point2}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct GridWindowIterator {
    next: Point2<i16>,
    range: Range<Point2<i16>>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

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

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl Iterator for GridWindowIterator {
    type Item = Point2<i16>;

    fn next(&mut self) -> Option<Point2<i16>> {
        if self.next.y < self.range.end.y {
            if self.next.x < self.range.end.x {
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
        } else {
            None
        }
    }
}

