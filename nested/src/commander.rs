
pub trait Commander {
    type Cmd;

    fn send_cmd(&mut self, cmd: &Self::Cmd);
}

use std::sync::{Arc, RwLock};
use crate::{
    type_system::ReprTree
};
use r3vi::view::singleton::*;

pub trait ObjCommander {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>);
}

//impl<Cmd: 'static, T: Commander<Cmd>> ObjCommander for T {
impl<C: Commander> ObjCommander for C
where C::Cmd: 'static
{
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        self.send_cmd(
            &cmd_obj.read().unwrap()
                .get_port::<dyn SingletonView<Item = C::Cmd>>().unwrap()
                .get_view().unwrap()
                .get()
        );
    }
}

