
pub trait Commander {
    type Cmd;

    fn send_cmd(&mut self, cmd: &Self::Cmd);
}

use std::sync::{Arc, RwLock};
use crate::{
    type_system::ReprTree,
    tree::{nav::TreeNavResult}
};

pub trait ObjCommander {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult;
}

