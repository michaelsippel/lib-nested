
use {
    laddertypes::{TypeTerm, TypeID},
    crate::{
        editTree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, addr: &TreeAddr) -> Vec<TypeTerm> {
        vec![]
    }
}

