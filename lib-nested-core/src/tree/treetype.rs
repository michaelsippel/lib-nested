
use {
    laddertypes::{TypeTerm, TypeID},
    crate::{
        tree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, addr: &TreeAddr) -> Vec<TypeTerm> {
        vec![]
    }
}

