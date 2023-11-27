use {
    r3vi::{
        view::{singleton::*}
    },
    crate::{
        reprTree::{Context, ReprTree},
        editTree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        editors::{list::{ListEditor, ListCmd, ListCursorMode}, ObjCommander},
    },
    std::{sync::{Arc, RwLock}},

    super::{TypeTermEditor, State}
};

impl ObjCommander for TypeTermEditor {
    fn send_cmd_obj(&mut self, co: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let _cur = self.get_cursor();

        let cmd_obj = co.clone();
        let cmd_obj = cmd_obj.read().unwrap();

        if cmd_obj.get_type().clone() == Context::parse(&self.ctx, "Char") {
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
                                self.set_state( State::FunSymbol );
                                self.cur_node.get_mut().goto(TreeCursor::home());
                                self.send_child_cmd( co )
                            }
                        }
                    }

                    State::Char => {
                        match self.send_child_cmd( co ) {
                            TreeNavResult::Exit => {
                                match c {
                                    '\'' => {
                                        self.cur_node.get_mut().goto(TreeCursor::none());
                                    }
                                    _ => {}
                                }
                                TreeNavResult::Exit
                            },
                            TreeNavResult::Continue => TreeNavResult::Continue
                        }
                    }

                    State::Ladder => {

                        match self.get_cursor().tree_addr.len() {

                            // entire term is selected
                            0 => {
                                match c {
                                    '<' => {
                                        self.morph_to_list(State::App);
                                        TreeNavResult::Continue
                                    }
                                    _ => { TreeNavResult::Exit }
                                }
                            }

                            // entire item selected or insert mode
                            1 => {
                                match c {
                                    '~' => {
                                        // ignore '~' since we are already in a ladder
                                        // and cant split current item
                                        TreeNavResult::Continue
                                    }
                                    _ => {
                                        self.send_child_cmd( co.clone() )
                                    }
                                }
                            }

                            // some subterm
                            _ => {
                                match c {
                                    '~' => {
                                        let i0 = self.cur_node.get().get_edit::<ListEditor>().unwrap();
                                        let cur_it = i0.clone().read().unwrap().get_item().clone();
                                        if let Some(i) = cur_it {
                                            let cur_tte = i.get_edit::<TypeTermEditor>().unwrap();
                                            if cur_tte.read().unwrap().state == State::App || cur_tte.read().unwrap().get_cursor().tree_addr.len() > 1 {
                                                self.send_child_cmd( co.clone() )
                                            } else {
                                                drop(cur_tte);
                                                drop(i);

                                                self.send_child_cmd(
                                                    ListCmd::Split.into_repr_tree( &self.ctx )
                                                )
                                            }
                                        } else {
                                            self.send_child_cmd( co.clone() )
                                        }
                                    }
                                    _ => {
                                        self.send_child_cmd( co.clone() )
                                    }
                                }
                            }
                        }
                    }

                    State::App => {

                        match self.get_cursor().tree_addr.len() {

                            // entire Term is selected
                            0 => {
                                
                                match c {
                                    '~' => {
                                        self.morph_to_list(State::Ladder);
                                        self.goto(TreeCursor {
                                            tree_addr: vec![ -1 ],
                                            leaf_mode: ListCursorMode::Insert
                                        });
                                        TreeNavResult::Continue
                                    }
                                    '<' => {
                                        self.morph_to_list(State::App);
                                        TreeNavResult::Continue
                                    }
                                    _ => {
                                        TreeNavResult::Exit
                                    }
                                }

                            },

                            // some item is selected
                            _ => {
                                match self.send_child_cmd( co.clone() ) {
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
                        }
                    }

                    State::AnySymbol |
                    State::FunSymbol |
                    State::VarSymbol => {
                        let res = self.send_child_cmd( co );
                        match res {
                            TreeNavResult::Exit => {
                                match c {
                                    '<' => {
                                        self.goto(TreeCursor::none());
                                        self.morph_to_list(State::App);
                                        TreeNavResult::Continue
                                    }
                                    '~' => {
                                        self.morph_to_list(State::Ladder);
                                        self.set_addr(0);
                                        self.dn();
                                        self.send_cmd_obj(
                                            ListCmd::Split.into_repr_tree( &self.ctx )
                                        );
                                        TreeNavResult::Continue
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
                    if cmd_repr.get_type().clone() == Context::parse(&self.ctx, "NestedNode") {
                        if let Some(view) = cmd_repr.get_view::<dyn SingletonView<Item = NestedNode>>() {
                            let node = view.get();

                            if node.data.read().unwrap().get_type().clone() == Context::parse(&self.ctx, "Char") {
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

