use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{
            ListCursor, ListCursorMode,
            ListSegment, ListSegmentSequence,
            ListEditor
        },
        sequence::SequenceView,
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            make_label, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
        vec::VecBuffer,
        color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    cgmath::Vector2
};

impl<ItemEditor> TreeNav for ListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    fn get_cursor_warp(&self) -> TreeCursor {
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert => TreeCursor {
                leaf_mode: cur.mode,
                tree_addr: if let Some(i) = cur.idx {
                    vec![
                        i - self.data.len() as isize - 1
                    ]
                } else {
                    vec![]
                },
            },
            ListCursorMode::Select => {
                if let Some(i) = cur.idx {
                    if i < self.data.len() as isize {
                        let mut sub_cur = self.data.get(i as usize).read().unwrap().get_cursor_warp();
                        sub_cur.tree_addr.insert(0, i as isize - self.data.len() as isize);
                        return sub_cur;
                    }
                }
                TreeCursor {
                    leaf_mode: cur.mode,
                    tree_addr: vec![],
                }
            }
        }
    }

    fn get_cursor(&self) -> TreeCursor {
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert => TreeCursor {
                leaf_mode: cur.mode,
                tree_addr: if let Some(i) = cur.idx {
                    vec![i]
                } else {
                    vec![]
                },
            },
            ListCursorMode::Select => {
                if let Some(i) = cur.idx {
                    if i < self.data.len() as isize {
                        let mut sub_cur = self.data.get(i as usize).read().unwrap().get_cursor();
                        sub_cur.tree_addr.insert(0, i as isize);
                        return sub_cur;
                    }
                }
                TreeCursor {
                    leaf_mode: cur.mode,
                    tree_addr: vec![],
                }
            }
        }
    }

    fn goto(&mut self, new_cur: TreeCursor) -> TreeNavResult {
        let old_cur = self.cursor.get();
        if let Some(i) = old_cur.idx {
            if i < self.data.len() as isize {
                let ce = self.data.get_mut(i as usize);
                let mut cur_edit = ce.write().unwrap();
                cur_edit.goto(TreeCursor::none());
            }
        }

        match new_cur.tree_addr.len() {
            0 => {
                self.cursor.set(ListCursor {
                    mode: new_cur.leaf_mode,
                    idx: None,
                });
                TreeNavResult::Continue
            }
            1 => {
                self.cursor.set(ListCursor {
                    mode: new_cur.leaf_mode,
                    idx: Some(crate::modulo(new_cur.tree_addr[0], if new_cur.leaf_mode == ListCursorMode::Insert { 1 } else { 0 } + self.data.len() as isize)),
                });
                TreeNavResult::Continue
            }
            _ => {
                let idx = crate::modulo(new_cur.tree_addr[0], self.data.len() as isize);

                self.cursor.set(ListCursor {
                    mode: ListCursorMode::Select,
                    idx: Some(idx),
                });

                let item = self.data.get_mut(idx as usize);
                let mut item_edit = item.write().unwrap();

                item_edit.goto(TreeCursor {
                    leaf_mode: new_cur.leaf_mode,
                    tree_addr: new_cur.tree_addr[1..].iter().cloned().collect(),
                });

                TreeNavResult::Continue                
            }
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        let mut cur = self.get_cursor();
        match cur.tree_addr.len() {
            0 => {

                if direction.y < 0 {
                    // up
                    TreeNavResult::Exit
                } else if direction.y > 0 {
                    // dn
                    self.cursor.set(ListCursor::home());
                    self.goby(Vector2::new(direction.x, direction.y-1));
                    TreeNavResult::Continue
                } else {
                    TreeNavResult::Continue
                }
            },

            1 => {
                if direction.y > 0 {
                    // dn
                    if cur.tree_addr[0] < self.data.len() as isize {
                        let item = self.data.get_mut(cur.tree_addr[0] as usize);
                        let mut item_edit = item.write().unwrap();

                        if item_edit.goby(Vector2::new(direction.x, direction.y)) == TreeNavResult::Continue {
                            self.cursor.set(ListCursor {
                                mode: ListCursorMode::Select,
                                idx: Some(cur.tree_addr[0])
                            })
                        }
                    }

                    TreeNavResult::Continue

                } else if direction.y < 0 {
                    // up

                    self.cursor.set(ListCursor::none());
                    TreeNavResult::Exit                    
                } else {
                    // horizontal

                    if (cur.tree_addr[0]+direction.x >= 0) &&
                        (cur.tree_addr[0]+direction.x <
                         self.data.len() as isize
                         + if cur.leaf_mode == ListCursorMode::Insert { 1 } else { 0 })
                    {
                        self.cursor.set(ListCursor {
                            mode: cur.leaf_mode,
                            idx: Some(cur.tree_addr[0] + direction.x)
                        });
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::none());
                        TreeNavResult::Exit
                    }
                }
            },
            depth => {
                // nested

                let item = self.data.get_mut(cur.tree_addr[0] as usize);
                let mut item_edit = item.write().unwrap();

                match item_edit.goby(direction) {
                    TreeNavResult::Exit => {
                        if direction.y < 0 {
                            // up
                            self.cursor.set(ListCursor {
                                mode: cur.leaf_mode,
                                idx: Some(cur.tree_addr[0])
                            });

                            TreeNavResult::Continue
                        } else if direction.y > 0 {
                            // dn

                            TreeNavResult::Continue
                        } else {
                            // horizontal
                            drop(item_edit);

                            if (cur.tree_addr[0]+direction.x >= 0) &&
                                (cur.tree_addr[0]+direction.x < self.data.len() as isize)
                            {
                                if direction.x < 0 {
                                    cur.tree_addr[0] -= 1;
                                    for i in 1..depth {
                                        cur.tree_addr[i] = -1;
                                    }
                                } else {
                                    cur.tree_addr[0] += 1;
                                    for i in 1..depth {
                                        cur.tree_addr[i] = 0;
                                    }
                                }

                                self.goto(cur)                                
                            } else {
                                self.cursor.set(ListCursor::none());
                                TreeNavResult::Exit
                            }
                        }
                    }
                    TreeNavResult::Continue => TreeNavResult::Continue
                }                
            }
        }
    }
}
