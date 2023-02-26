
use {
    crate::{
        type_system::TypeLadder,
        tree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, _addr: &TreeAddr) -> TypeLadder {
        vec![].into()
    }
}

