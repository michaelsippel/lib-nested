
use {
    laddertypes::{TypeTerm, TypeID},
    crate::{
        edit_tree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, addr: &TreeAddr) -> Vec<TypeTerm> {
        vec![]
    }
}

