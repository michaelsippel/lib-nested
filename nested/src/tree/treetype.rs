
use {
    crate::{
        type_system::{TypeTerm, TypeID},
        tree::{TreeAddr}
    }
};

pub trait TreeType {
    fn get_type(&self, _addr: &TreeAddr) -> TypeTerm {
        TypeTerm::new(TypeID::Var(0))
    }
}

