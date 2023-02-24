
pub trait Commander {
    type Cmd;

    fn send_cmd(&mut self, cmd: &Self::Cmd);
}

use std::sync::{Arc, RwLock};
use crate::{
    type_system::ReprTree,
    tree::{nav::TreeNavResult}
};

//use r3vi::view::singleton::*;

pub trait ObjCommander {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult;
}
/*
//impl<Cmd: 'static, T: Commander<Cmd>> ObjCommander for T {
impl<C: Commander> ObjCommander for C
where C::Cmd: 'static
{
    fn send_cmd_obj(&mut self, _cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult{
        /*
        self.send_cmd(
            &cmd_obj.read().unwrap()
                .get_port::<dyn SingletonView<Item = C::Cmd>>().unwrap()
                .get_view().unwrap()
                .get()
    );
        */
    }
}

impl<T: Clone + Send + Sync> Commander for r3vi::buffer::vec::VecBuffer<T> {
    type Cmd = r3vi::buffer::vec::VecDiff<T>;

    fn send_cmd(&mut self, cmd: &Self::Cmd) -> TreeNavResult {
        self.apply_diff(cmd.clone());
        TreeNavResult::
    }
}
*/
