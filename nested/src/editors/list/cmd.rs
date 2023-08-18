use {
    r3vi::{
        view::{singleton::*},
        buffer::{singleton::*}
    },
    crate::{
        editors::list::{ListEditor, ListCursorMode},
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
    pub fn into_repr_tree(self, ctx: &Arc<RwLock<Context>>) -> Arc<RwLock<ReprTree>> {
        let buf = r3vi::buffer::singleton::SingletonBuffer::new(self);
        ReprTree::new_leaf(
            (ctx, "( ListCmd )"),
            buf.get_port().into()
        )
    }
}

impl ObjCommander for ListEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_repr = cmd_obj.read().unwrap();

        if let Some(cmd) = cmd_repr.get_view::<dyn SingletonView<Item = ListCmd>>() {
            let cur = self.cursor.get();
            drop(cmd_repr);
            
            if let Some(idx) = cur.idx {
                match cur.mode {
                    ListCursorMode::Select => {
                        if let Some(mut item) = self.get_item().clone() {
                            if self.is_listlist() {
                                let item_cur = item.get_cursor();

                                match cmd.get() {
                                    ListCmd::DeletePxev => {
                                        if idx > 0
                                            && item_cur.tree_addr.iter().fold(
                                                true,
                                                |is_zero, x| is_zero && (*x == 0)
                                            )
                                        {
                                            self.listlist_join_pxev(idx, &item);
                                            TreeNavResult::Continue
                                        } else {
                                            item.send_cmd_obj(cmd_obj)
                                        }
                                    }

                                    ListCmd::DeleteNexd => {
                                        let item_cur = item.get_cursor_warp();
                                        let next_idx = idx as usize + 1;

                                        if next_idx < self.data.len()
                                            && item_cur.tree_addr.iter().fold(
                                                true,
                                                |is_end, x| is_end && (*x == -1)
                                            )
                                        {
                                            self.listlist_join_nexd(next_idx, &item);
                                            TreeNavResult::Continue
                                        } else {
                                            item.send_cmd_obj(cmd_obj)
                                        }
                                    }

                                    _ => {
                                        TreeNavResult::Continue
                                    }
                                }
                            } else {
                                TreeNavResult::Exit
                            }
                        } else {
                            TreeNavResult::Exit
                        }
                    },

                    ListCursorMode::Insert => {
                        match cmd.get() {
                            ListCmd::DeletePxev => {
                                self.delete_pxev();
                                TreeNavResult::Continue
                            }
                            ListCmd::DeleteNexd => {
                                self.delete_nexd();
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
                            _ =>{
                                TreeNavResult::Continue
                            }
                        }
                    }
                }
            } else {
                TreeNavResult::Exit
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

