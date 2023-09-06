use {
    r3vi::{
        view::{singleton::*}
    },
    crate::{
        type_system::{ReprTree},
        editors::{list::{ListEditor, ListCmd}},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander
    },
    std::{sync::{Arc, RwLock}},

    super::{TypeTermEditor, State}
};

impl ObjCommander for TypeTermEditor {
    fn send_cmd_obj(&mut self, co: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let _cur = self.get_cursor();

        let cmd_obj = co.clone();
        let cmd_obj = cmd_obj.read().unwrap();

        if cmd_obj.get_type().clone() == (&self.ctx, "( Char )").into() {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let c = cmd_view.get();

                match &self.state {
                    State::Any => {
                        match c {
                            '<' => {
                                self.set_state( State::App );
                                TreeNavResult::Continue
                            }
                            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                                self.set_state( State::Num );
                                self.send_child_cmd( co );
                                TreeNavResult::Continue
                            }
                            '\'' => {
                                self.set_state( State::Char );
                                TreeNavResult::Continue
                            }
                            '~' => {
                                TreeNavResult::Exit
                            }
                            _ => {
                                self.set_state( State::AnySymbol );
                                self.cur_node.get_mut().goto(TreeCursor::home());
                                self.send_child_cmd( co )
                            }
                        }
                    }

                    State::Char => {
                        match c {
                            '\'' => {
                                self.cur_node.get_mut().goto(TreeCursor::none());
                                TreeNavResult::Exit
                            }
                            _ => {
                                self.send_child_cmd( co )
                            }
                        }
                    }

                    State::Ladder => {
                        let res = self.send_child_cmd( co.clone() );
                        
                        match res {
                            TreeNavResult::Continue => {
                                match c {
                                    '~' => {
                                        self.normalize_nested_ladder();
                                    }
                                    _ => {}
                                }
                                TreeNavResult::Continue
                            }
                            TreeNavResult::Exit => {
                                match c {
                                    '~' => TreeNavResult::Continue,
                                    _   => TreeNavResult::Exit
                                }
                            }
                        }
                    }

                    State::App => {
                        let res = self.send_child_cmd( co.clone() );

                        match res {
                            TreeNavResult::Exit => {
                                match c {
                                    '~' => {
                                        self.previous_item_into_ladder();
                                        TreeNavResult::Continue
                                    },
                                    _ => {TreeNavResult::Exit}
                                }
                            },
                            TreeNavResult::Continue => {
                                match c {
                                    '>'|
                                    ' ' => {
                                        let i = self.cur_node.get().get_edit::<ListEditor>().unwrap();
                                        let i = i.read().unwrap();
                                        if let Some(i) = i.get_item() {
                                            let tte = i.get_edit::<TypeTermEditor>().unwrap();
                                            let mut tte = tte.write().unwrap();

                                            if tte.state == State::Ladder {
                                                tte.normalize_singleton();
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                                
                                TreeNavResult::Continue
                            }
                        }
                    }

                    State::AnySymbol |
                    State::FunSymbol |
                    State::VarSymbol => {
                        let res = self.send_child_cmd( co );
                        match res {
                            TreeNavResult::Exit => {
                                match c {
                                    '~' => {
                                        self.morph_to_ladder();
                                        self.send_cmd_obj(
                                            ListCmd::Split.into_repr_tree( &self.ctx )
                                        )
                                    }
                                    _ => {
                                        TreeNavResult::Exit
                                    }
                                }
                            }
                            TreeNavResult::Continue => {
                                TreeNavResult::Continue
                            }
                        }
                    }

                    _ => {
                        self.send_child_cmd( co )
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            match &self.state {
                State::Any => {
                    let cmd_repr = co.read().unwrap();
                    if cmd_repr.get_type().clone() == (&self.ctx, "( NestedNode )").into() {
                        if let Some(view) = cmd_repr.get_view::<dyn SingletonView<Item = NestedNode>>() {
                            let node = view.get();

                            if node.data.read().unwrap().get_type().clone() == (&self.ctx, "( Char )").into() {
                                self.set_state( State::AnySymbol );
                            } else {
                                self.set_state( State::Ladder );
                            }
                        } else {
                            eprintln!("ERROR");
                        }
                    } else {
                        self.set_state( State::AnySymbol );
                    }

                    self.cur_node.get_mut().goto(TreeCursor::home());
                }
                State::Ladder | State::App => {
                    // todo: if backspace cmd and empty list, reset to Any
                }
                _ => {
                }
            }

            let res = self.send_child_cmd( co.clone() );

            self.normalize_empty();

            if let Some(cmd) = co.read().unwrap().get_view::<dyn SingletonView<Item = ListCmd>>() {
                match cmd.get() {
                    ListCmd::Split => {
                        if self.state == State::Ladder {
                            self.normalize_singleton();
                        }
                    }
                    _ =>{}
                }
            }

            res
        }
    }
}

