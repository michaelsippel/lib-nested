
pub mod list;
//pub mod product;
//pub mod sum;

pub mod char;
//pub mod integer;
//pub mod typeterm;


pub trait Commander {
    type Cmd;

    fn send_cmd(&mut self, cmd: &Self::Cmd);
}

use std::sync::{Arc, RwLock};
use crate::{
    repr_tree::ReprTree,
    edit_tree::nav::TreeNavResult
};

pub trait ObjCommander {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult;
}

