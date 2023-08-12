use {
    r3vi::{
        view::{singleton::*},
        buffer::{singleton::*}
    },
    crate::{
        editors::list::ListEditor,
        type_system::{Context, ReprTree},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::{ObjCommander}
    },
    std::sync::{Arc, RwLock}
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ListCmd {    
    DeletePxev,
    DeleteNexd,
    JoinNexd,
    JoinPxev,
    Split,
    Clear,
    Close,
}

impl ListCmd {
    fn into_repr_tree(self, ctx: &Arc<RwLock<Context>>) -> Arc<RwLock<ReprTree>> {
        let buf = r3vi::buffer::singleton::SingletonBuffer::new(self);
        ReprTree::new_leaf(
            (ctx, "( ListCmd )"),
            buf.get_port().into()
        )
    }
}
/*
impl Into< Arc<RwLock<ReprTree>> > for (&Arc<RwLock<Context>>, ListCmd) {
    fn into(self) -> Arc<RwLock<ReprTree>> {
        self.1.into_repr_tree(self.0)
    }
}
*/
impl ObjCommander for ListEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_repr = cmd_obj.read().unwrap();
        if let Some(cmd) = cmd_repr.get_view::<dyn SingletonView<Item = ListCmd>>() {
            match cmd.get() {
                ListCmd::DeletePxev => {
                    self.delete_pxev();
                    TreeNavResult::Continue
                }
                ListCmd::DeleteNexd => {
                    self.delete_nexd();
                    TreeNavResult::Continue
                }
                ListCmd::JoinPxev => {
                    // TODO
                    //self.listlist_join_pxev();
                    TreeNavResult::Continue
                }
                ListCmd::JoinNexd => {
                    // TODO
                    //self.listlist_join_nexd();
                    TreeNavResult::Continue
                }
                ListCmd::Split => {
                    self.listlist_split();
                    TreeNavResult::Continue
                }
                ListCmd::Clear => {
                    self.clear();
                    TreeNavResult::Continue
                }
                ListCmd::Close => {
                    self.goto(TreeCursor::none());
                    TreeNavResult::Exit
                }
            }
        } else {
            if let Some(cur_item) = self.get_item_mut() {
                drop(cmd_repr);
                cur_item.write().unwrap().send_cmd_obj(cmd_obj)
            } else {
                TreeNavResult::Continue
            }
        }
    }
}

