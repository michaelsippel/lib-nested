use {
    crate::{
        core::Context,
        list::ListCursorMode,
        tree::{TreeNav, TreeNavResult, TreeCursor},
        product::{segment::ProductEditorSegment, ProductEditor},
        Nested
    },
    cgmath::{Vector2},
    std::{ops::{DerefMut}},
};

impl TreeNav for ProductEditor {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(i) = self.cursor {
            if let Some(e) = self.get_editor(i) {
                let mut c = e.read().unwrap().get_cursor();
                if c.tree_addr.len() == 0 {
                    c.leaf_mode = ListCursorMode::Select;
                }
                c.tree_addr.insert(0, i);                
                c
            } else {
                TreeCursor {
                    leaf_mode: ListCursorMode::Select,
                    tree_addr: vec![ i ]
                }
            }
        } else {
            TreeCursor::none()
        }
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        if let Some(i) = self.cursor {
            if let Some(e) = self.get_editor(i) {
                let mut c = e.read().unwrap().get_cursor_warp();
                if c.tree_addr.len() == 0 {
                    c.leaf_mode = ListCursorMode::Select;
                }
                c.tree_addr.insert(0, i as isize - self.n_indices.len() as isize);
                c
            } else {
                TreeCursor {
                    leaf_mode: ListCursorMode::Select,
                    tree_addr: vec![ i as isize - self.n_indices.len() as isize ]
                }
            }
        } else {
            TreeCursor::none()
        }
    }

    fn goto(&mut self, mut c: TreeCursor) -> TreeNavResult {
        let old_cursor = self.cursor;

        if let Some(mut segment) = self.get_cur_segment_mut() {
            if let Some(ProductEditorSegment::N{ t: _t, editor, ed_depth: _, cur_depth: _, cur_dist:_ }) = segment.deref_mut() {
                if let Some(e) = editor {
                    let mut e = e.write().unwrap();
                    e.goto(TreeCursor::none());
                }
            }
        }

        if c.tree_addr.len() > 0 {
            self.cursor = Some(crate::modulo(c.tree_addr.remove(0), self.n_indices.len() as isize));
            if let Some(mut element) = self.get_cur_segment_mut() {
                if let Some(ProductEditorSegment::N{ t, editor, ed_depth, cur_depth: _, cur_dist:_ }) = element.deref_mut() {
                    if let Some(e) = editor {
                        e.write().unwrap().goto(c.clone());
                    } else if c.tree_addr.len() > 0 {
                        // create editor
                        let e = Context::make_editor(self.ctx.clone(), t[0].clone(), *ed_depth+1).unwrap();
                        *editor = Some(e.clone());
                        let mut e = e.write().unwrap();
                        e.goto(c.clone());
                    }
                }
            }

            if let Some(i) = old_cursor{
                self.update_segment(i);
            }

            if let Some(i) = self.cursor {
                self.update_segment(i);
            }

            TreeNavResult::Continue
        } else {
            if let Some(ed) = self.get_cur_editor() {
                ed.write().unwrap().goto(TreeCursor::none());
            }

            self.cursor = None;

            if let Some(i) = old_cursor {
                self.update_segment(i);
            }
            TreeNavResult::Exit
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        let mut cur = self.get_cursor();
        
        match cur.tree_addr.len() {
            0 => {
                if direction.y > 0 {
                    self.cursor = Some(0);
                    self.update_segment(0);

                    self.goby(Vector2::new(direction.x, direction.y-1));
                    TreeNavResult::Continue
                } else if direction.y < 0 {
                    TreeNavResult::Exit
                } else {
                    TreeNavResult::Continue
                }
            }
            1 => {
                if direction.y > 0 {
                    // dn
                    if let Some(mut element) = self.get_cur_segment_mut() {
                        if let Some(ProductEditorSegment::N{ t, editor, ed_depth, cur_depth: _, cur_dist:_ }) = element.deref_mut() {
                            if let Some(e) = editor {
                                let mut e = e.write().unwrap();
                                e.goby(direction);
                            } else {
                                // create editor

                                let e = Context::make_editor(self.ctx.clone(), t[0].clone(), *ed_depth+1).unwrap();
                                *editor = Some(e.clone());
                                let mut e = e.write().unwrap();
                                e.goby(direction);
                            }
                        }
                    }

                    self.update_segment(cur.tree_addr[0]);
                    TreeNavResult::Continue
                } else if direction.y < 0 {
                    // up
                    let old_cursor = self.cursor;
                    self.cursor = None;
                    if let Some(i) = old_cursor {
                        self.update_segment(i);
                    }
                    TreeNavResult::Exit
                } else {
                    let old_cursor = self.cursor;

                    if (cur.tree_addr[0]+direction.x >= 0) &&
                        (cur.tree_addr[0]+direction.x < self.n_indices.len() as isize)
                    {
                        self.cursor = Some(cur.tree_addr[0] + direction.x);

                        self.update_cur_segment();
                        if let Some(i) = old_cursor {
                            self.update_segment(i);
                        }
                        TreeNavResult::Continue
                    } else {
                        self.cursor = None;
                        if let Some(i) = old_cursor {
                            self.update_segment(i);
                        }
                        TreeNavResult::Exit
                    }
                }
            }
            depth => {
                let old_cursor = self.cursor;
                let nav_result =
                    if let Some(mut element) = self.get_cur_segment_mut() {
                        if let Some(ProductEditorSegment::N{ t: _, editor, ed_depth: _, cur_depth, cur_dist:_ }) = element.deref_mut() {
                            if let Some(e) = editor {
                                let mut ce = e.write().unwrap();
                                //\\//\\//\\//\\
                                // horizontal //
                                //\\//\\//\\//\\
                                match ce.goby(direction) {
                                    TreeNavResult::Exit => {
                                       // *cur_depth = 1;
                                        drop(ce);
                                        drop(e);

                                        if direction.y < 0 {
                                            if depth <= (1-direction.y) as usize {
                                                // up
                                                TreeNavResult::Continue
                                            } else {
                                                panic!("unplausible direction.y on exit");
                                            }
                                        } else if direction.y > 0 {
                                            // dn
                                            TreeNavResult::Continue
                                        } else if direction.y == 0 {
                                            // horizontal
                                            if direction.x != 0 {
                                                *cur_depth = 0;
                                            }

                                            if (cur.tree_addr[0]+direction.x >= 0) &&
                                                (cur.tree_addr[0]+direction.x < self.n_indices.len() as isize)
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
                                                self.cursor = None;
                                                TreeNavResult::Exit
                                            }
                                        } else {
                                            TreeNavResult::Continue
                                        }
                                    }
                                    TreeNavResult::Continue => {
                                        TreeNavResult::Continue
                                    }
                                }
                            } else {
                                TreeNavResult::Continue
                            }
                        } else {
                            TreeNavResult::Continue
                        }
                    } else {
                        TreeNavResult::Continue
                    };

                if let Some(i) = old_cursor {
                    self.update_segment(i);
                }

                self.update_cur_segment();
                return nav_result;
            }
        }
    }
}

impl Nested for ProductEditor {}

