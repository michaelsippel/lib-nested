pub mod singleton;
pub mod index;
pub mod sequence;

pub use {
    singleton::SingletonView,
    index::{IndexView, ImplIndexView},
    sequence::SequenceView,
    crate::core::View
};

