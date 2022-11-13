pub mod cursor;
pub mod nav;
pub mod typeinfo;

pub struct TreeAddr(Vec<usize>);

impl From<Vec<usize>> for TreeAddr {
    fn from(v: Vec<usize>) -> TreeAddr {
        TreeAddr(v)
    }
}

pub use {
    cursor::TreeCursor,
    nav::{TreeNav, TreeNavResult},
    typeinfo::TreeType
};

