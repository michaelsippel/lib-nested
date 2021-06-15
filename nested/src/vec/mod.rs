
pub mod buffer;
pub mod vec2seq;
pub mod vec2json;
pub mod vec2bin;

pub use {
    buffer::VecBuffer
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use {
    serde::{Serialize, Deserialize}
};

#[derive(Clone, Serialize, Deserialize)]
pub enum VecDiff<T> {
    Clear,
    Push(T),
    Remove(usize),
    Insert{ idx: usize, val: T },
    Update{ idx: usize, val: T }
}

