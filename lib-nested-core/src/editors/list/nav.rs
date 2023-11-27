use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
            sequence::*
        }
    },
    crate::{
        editors::list::{
            ListCursor, ListCursorMode,
            editor::ListEditor
        },
        editTree::{TreeCursor, TreeNav, TreeNavResult, TreeHeightOp}
    },
    cgmath::Vector2
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TreeNav for ListEditor {
    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        self.addr_port.clone()
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        self.mode_port.clone()
    }
    
    fn get_height(&self, op: &TreeHeightOp) -> usize {
        match op {
            TreeHeightOp::P | TreeHeightOp::Q => {
                if self.data.len() > 0 {
                    1 + self.data.get(match op {
                        TreeHeightOp::P => 0,
                        TreeHeightOp::Q => self.data.len() - 1,
                        _ => 0
                    }).read().unwrap().get_height(op)
                } else {
                    1
                }
            }
            TreeHeightOp::Max => {
                1 + (0..self.data.len() as usize)
                    .map(|i| self.data
                         .get(i).read().unwrap()
                         .get_height(&TreeHeightOp::Max)
                    )
                    .max()
                    .unwrap_or(0)
            }
        }
    }

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
                }
            },
            ListCursorMode::Select => {
                if let Some(i) = cur.idx {
                    if i < self.data.len() as isize {
                        let mut sub_cur = self.data.get(i as usize).read().unwrap().get_cursor_warp();
                        sub_cur.tree_addr.insert(0, i as isize - self.data.len() as isize);
                        return sub_cur;
                    } else {
                        return TreeCursor {
                            leaf_mode: ListCursorMode::Select,
                            tree_addr: vec![ i as isize - self.data.len() as isize ],
                        };
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
                        if sub_cur.tree_addr.len() > 0 {
                            sub_cur.tree_addr.insert(0, i as isize);
                            return sub_cur;
                        } else {                            
                            return TreeCursor {
                                leaf_mode: ListCursorMode::Select,
                                tree_addr: vec![ i ],
                            };
                        }
                    }
                }
                TreeCursor {
                    leaf_mode: ListCursorMode::Select,
                    tree_addr: vec![],
                }
            }
        }
    }

    fn goto(&mut self, new_cur: TreeCursor) -> TreeNavResult {
        let old_cur = self.cursor.get();
        if let Some(i) = old_cur.idx {
            if i < self.data.len() as isize {
                self.data.get_mut(i as usize).write().unwrap().goto(TreeCursor::none());
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
                let idx = crate::utils::modulo(new_cur.tree_addr[0], if new_cur.leaf_mode == ListCursorMode::Insert { 1 } else { 0 } + self.data.len() as isize);

                self.cursor.set(ListCursor {
                    mode: new_cur.leaf_mode,
                    idx: Some(idx),
                });

                if new_cur.leaf_mode == ListCursorMode::Select && self.data.len() > 0 {
                    self.data
                        .get_mut(idx as usize)
                        .write().unwrap()
                        .goto(TreeCursor {
                            leaf_mode: ListCursorMode::Select,
                            tree_addr: vec![]
                        });
                }

                TreeNavResult::Continue
            }
            _ => {
                if self.data.len() > 0 {
                    let idx = crate::utils::modulo(new_cur.tree_addr[0], self.data.len() as isize);

                    self.cursor.set(ListCursor {
                        mode: ListCursorMode::Select,
                        idx: Some(idx),
                    });

                    self.data
                        .get_mut(idx as usize)
                        .write().unwrap()
                        .goto(TreeCursor {
                            leaf_mode: new_cur.leaf_mode,
                            tree_addr: new_cur.tree_addr[1..].iter().cloned().collect(),
                        });
                } else {
                    self.cursor.set(ListCursor::home());
                }

                TreeNavResult::Continue                
            }
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        let mut cur = self.get_cursor();

        let gravity = true;

        match cur.tree_addr.len() {
            0 => {

                if direction.y < 0 {
                    // up
                    self.cursor.set(ListCursor::none());
                    TreeNavResult::Exit
                } else if direction.y > 0 {
                    // dn
                    eprintln!("dn: data.len() = {}", self.data.len());
                    self.cursor.set(ListCursor {
                        mode: if self.data.len() > 0 { cur.leaf_mode } else { ListCursorMode::Insert },
                        idx: Some(0)
                    });

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
                        if self.data
                            .get_mut(cur.tree_addr[0] as usize)
                            .write().unwrap()
                            .goby(Vector2::new(direction.x, direction.y))
                            == TreeNavResult::Continue {
                                self.cursor.set(ListCursor {
                                    mode: ListCursorMode::Select,
                                    idx: Some(cur.tree_addr[0])
                                });
                                self.set_leaf_mode(cur.leaf_mode);
                            }
                    }

                    TreeNavResult::Continue

                } else if direction.y < 0 {
                    // up
                    self.cursor.set(ListCursor {
                        mode: cur.leaf_mode,
                        idx: None
                    });
                    TreeNavResult::Exit
                } else {
                    // horizontal

                    if (cur.tree_addr[0]+direction.x >= 0) &&
                        (cur.tree_addr[0]+direction.x <
                         self.data.len() as isize
                         + if cur.leaf_mode == ListCursorMode::Insert { 1 } else { 0 })
                    {
                        let idx = cur.tree_addr[0] + direction.x;
                        let mut new_addr = Vec::new();

                        match cur.leaf_mode {
                            ListCursorMode::Select => {
                                let cur_item = self.data.get(cur.tree_addr[0] as usize);
                                let cur_height = cur_item.read().unwrap().get_height(&TreeHeightOp::Max);

                                let new_item = self.data
                                    .get_mut(idx as usize);

                                let height = new_item.read().unwrap().get_height(
                                    if direction.x < 0 {
                                        &TreeHeightOp::Q
                                    } else {
                                        &TreeHeightOp::P
                                    }
                                );

                                new_addr.push(idx);
                                if gravity && cur_height < 2 {
                                    for _ in 1..height {
                                        new_addr.push( if direction.x >= 0 {
                                            0
                                        } else {
                                            -1
                                        });
                                    }
                                }
                            }
                            ListCursorMode::Insert => {
                                let gravity = false;
                                if direction.x > 0
                                {
                                    if (cur.tree_addr[0] as usize) < self.data.len() {
                                        let cur_item = self.data.get(cur.tree_addr[0] as usize);
                                        let cur_height = cur_item.read().unwrap().get_height(&TreeHeightOp::P);

                                        if gravity && cur_height > 1 {
                                            new_addr.push( cur.tree_addr[0] );
                                            new_addr.push(0);
                                        } else {
                                            new_addr.push( idx );           
                                        }
                                    }
                                } else {
                                    if (idx as usize) < self.data.len() {
                                        let pxv_item = self.data.get(idx as usize);
                                        let pxv_height = pxv_item.read().unwrap().get_height(&TreeHeightOp::P);

                                        if gravity && pxv_height > 1 {
                                            new_addr.push( idx );
                                            new_addr.push( -1 );
                                        } else {
                                            new_addr.push( idx );           
                                        }
                                    }                                    
                                }
                            }
                        }

                        if self.data.len() == 0 {
                            cur.leaf_mode = ListCursorMode::Insert
                        }
                        cur.tree_addr = new_addr;
                        self.goto(cur);

                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor {
                            mode: cur.leaf_mode,
                            idx: None
                        });
                        self.cursor.set(ListCursor::none());
                        TreeNavResult::Exit
                    }
                }
            },
            depth => {
                // nested

                if cur.tree_addr[0] < self.data.len() as isize {

                    let cur_item = self.data
                        .get_mut(cur.tree_addr[0] as usize);

                    let result = cur_item.write().unwrap().goby(direction);

                    match result
                    {
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
                                if (cur.tree_addr[0]+direction.x >= 0) &&
                                    (cur.tree_addr[0]+direction.x < self.data.len() as isize)
                                {
                                    let mut new_addr = Vec::new();

                                    if direction.x < 0 {
                                        let pxv_item = self.data
                                            .get_mut(cur.tree_addr[0] as usize - 1);

                                        let pxv_height = pxv_item.read().unwrap().get_height(&TreeHeightOp::Q) as isize;
                                        let cur_height = cur_item.read().unwrap().get_height(&TreeHeightOp::P) as isize;
                                        let dist_from_ground = cur_height - (depth as isize - 1);
                                        let n_steps_down =
                                            if gravity {
                                                pxv_height - dist_from_ground
                                            } else {
                                                depth as isize - 1
                                            };

                                        eprintln!("<- LEFT CROSS: pxv_height = {}, cur_height = {}, dist_from_ground = {}, n_steps_down = {}", pxv_height, cur_height, dist_from_ground, n_steps_down);
                                        new_addr.push( cur.tree_addr[0] - 1 );
                                        for _i in 0..n_steps_down {
                                            new_addr.push( -1 );
                                        }
                                        
                                    } else {
                                        let nxd_item = self.data
                                            .get_mut(cur.tree_addr[0] as usize + 1);

                                        let cur_height = cur_item.read().unwrap().get_height(&TreeHeightOp::Q) as isize;
                                        let nxd_height = nxd_item.read().unwrap().get_height(&TreeHeightOp::P) as isize;
                                        let dist_from_ground = cur_height - (depth as isize - 1);
                                        let n_steps_down =
                                            if gravity {
                                                nxd_height - dist_from_ground
                                            } else {
                                                depth as isize - 1
                                            };

                                        eprintln!("-> RIGHT CROSS: cur_height = {}, nxd_height = {}, dist_from_ground = {}, n_steps_down = {}", cur_height, nxd_height, dist_from_ground, n_steps_down);
                                        new_addr.push( cur.tree_addr[0] + 1 );
                                        for _i in 0..n_steps_down {
                                            new_addr.push( 0 );
                                        }
                                    }

                                    drop(cur_item);

                                    eprintln!("CROSS: goto {:?}", new_addr);
                                    cur.tree_addr = new_addr;
                                    self.goto(cur)
                                } else {
                                    self.cursor.set(ListCursor {
                                        mode: cur.leaf_mode,
                                        idx: None
                                    });
                                    self.cursor.set(ListCursor::none());
                                    TreeNavResult::Exit
                                }
                            }
                        }
                        TreeNavResult::Continue => TreeNavResult::Continue
                    }


                } else {
                    self.cursor.set(
                        ListCursor {
                            mode: ListCursorMode::Insert,
                            idx: Some(0)
                        }
                    );
                    TreeNavResult::Continue
                }
            }
        }
    }
}

