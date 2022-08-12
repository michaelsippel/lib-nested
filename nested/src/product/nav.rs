use {
    crate::{
        list::ListCursorMode,
        tree_nav::{TreeNav, TreeNavResult, TreeCursor, TerminalTreeEditor},
        product::{segment::ProductEditorSegment, ProductEditor},
        make_editor::{make_editor}
    },
    cgmath::{Point2, Vector2},
    std::{sync::{Arc, RwLock}, ops::{Deref, DerefMut}},
    termion::event::{Event, Key},
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
        if let Some(mut segment) = self.get_cur_segment_mut() {
            if let Some(ProductEditorSegment::N{ t: _t, editor, cur_depth }) = segment.deref_mut() {
                if let Some(e) = editor {
                    let mut e = e.write().unwrap();
                    e.goto(TreeCursor::none());
                }
                *cur_depth = 0;
            }
        }

        if c.tree_addr.len() > 0 {
            self.cursor = Some(crate::modulo(c.tree_addr.remove(0), self.n_indices.len() as isize));

            if let Some(mut element) = self.get_cur_segment_mut() {
                if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                    if let Some(e) = editor {
                        e.write().unwrap().goto(c.clone());
                    } else if c.tree_addr.len() > 0 {
                        // create editor
                        let e = make_editor(self.ctx.clone(), t, self.depth+1);
                        *editor = Some(e.clone());
                        let mut e = e.write().unwrap();
                        e.goto(c.clone());
                    }
                    *cur_depth = c.tree_addr.len();
                }
            }

            TreeNavResult::Continue
        } else {
            self.cursor = None;
            TreeNavResult::Exit
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        let mut cur = self.get_cursor();
        
        match cur.tree_addr.len() {
            0 => {
                if direction.y > 0 {
                    self.cursor = Some(0);

                    if let Some(mut element) = self.get_cur_segment_mut() {
                        if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                            *cur_depth = 1;
                        }
                    }

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
                        if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                            if let Some(e) = editor {
                                let mut e = e.write().unwrap();
                                e.goby(direction);
                                *cur_depth = e.get_cursor().tree_addr.len() + 1;
                            } else {
                                // create editor
                                let e = make_editor(self.ctx.clone(), t, self.depth+1);
                                *editor = Some(e.clone());
                                let mut e = e.write().unwrap();
                                e.goby(direction);
                                *cur_depth = e.get_cursor().tree_addr.len() + 1;
                            }
                        }
                    }
                    TreeNavResult::Continue
                } else if direction.y < 0 {
                    // up
                    if let Some(mut element) = self.get_cur_segment_mut() {
                        if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                            *cur_depth = 0;
                        }
                    }
                    self.cursor = None;
                    TreeNavResult::Exit
                } else {
                    if let Some(mut element) = self.get_cur_segment_mut() {
                        if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                            *cur_depth = 0;
                        }
                    }

                    // horizontal
                    if (cur.tree_addr[0]+direction.x >= 0) &&
                        (cur.tree_addr[0]+direction.x < self.n_indices.len() as isize)
                    {
                        if let Some(mut element) = self.get_cur_segment_mut() {
                            if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                                *cur_depth = 0;
                            }
                        }

                        self.cursor = Some(cur.tree_addr[0] + direction.x);
                        if let Some(mut element) = self.get_cur_segment_mut() {
                            if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                                *cur_depth = 1;
                            }
                        }
                        TreeNavResult::Continue
                    } else {
                        self.cursor = None;
                        TreeNavResult::Exit
                    }
                }
            }
            depth => {
                if let Some(mut element) = self.get_cur_segment_mut() {
                    if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = element.deref_mut() {
                        if let Some(e) = editor {
                            let mut ce = e.write().unwrap();
                            //\\//\\//\\//\\
                            // horizontal //
                            //\\//\\//\\//\\
                            match ce.goby(direction) {
                                TreeNavResult::Exit => {
                                    *cur_depth = 1;
                                    drop(ce);
                                    drop(e);

                                    if direction.y < 0 {
                                        if depth <= (1-direction.y) as usize {
                                            // up
                                            *cur_depth = 1;
                                            TreeNavResult::Continue
                                        } else {
                                            panic!("unplausible direction.y on exit");
                                            TreeNavResult::Continue
                                        }
                                    } else if direction.y > 0 {
                                        // dn
                                        *cur_depth = depth + direction.y as usize;

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
                                    *cur_depth = (depth as isize + direction.y - 1) as usize;
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
                }
            }
        }
    }
}

impl TerminalTreeEditor for ProductEditor {}

