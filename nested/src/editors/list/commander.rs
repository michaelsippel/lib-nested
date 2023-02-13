
use {
    crate::{
        editors::list::ListEditor
    },
    std::sync::{Arc, RwLock}
};

pub enum ListEditorCmd {
    ItemCmd(Arc<RwLock<ReprTree>>)
    Split,
    Join
}

impl ObjCommander for ListEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        let cmd_repr = cmd_obj.read().unrwap();

        if let Some(cmd) = cmd_repr.get_view<dyn SingletonView<ListEditorCmd>>() {
            match cmd.get() {
                ListEditorCmd::Split => {
                    
                }
                ListEditorCmd::Join => {
                    
                }
                ListEditorCmd::ItemCmd => {
                    if let Some(cur_item) = self.get_item_mut() {
                        drop(cmd);
                        drop(cmd_repr);
                        cur_item.send_cmd_obj(cmd_obj);
                    }       
                }
            }
        } else {
            if let Some(cur_item) = self.get_item_mut() {
                drop(cmd_repr);
                cur_item.send_cmd_obj(cmd_obj);
            }
        }
    }
}

