pub mod buffer;
pub mod vec2bin;
pub mod vec2json;
pub mod vec2seq;

pub use buffer::{VecBuffer, MutableVecAccess};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum VecDiff<T> {
    Clear,
    Push(T),
    Remove(usize),
    Insert { idx: usize, val: T },
    Update { idx: usize, val: T },
}
