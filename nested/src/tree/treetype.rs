
use {
    crate::{
        type_system::{TypeTerm, TypeID},
        tree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, addr: &TreeAddr) -> Vec<TypeTerm> {
        vec![]
    }
}

