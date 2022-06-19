use {
    crate::{
        list::ListCursorMode,
        tree_nav::{TreeNav, TreeNavResult, TreeCursor, TerminalTreeEditor},
        product::{element::ProductEditorElement, ProductEditor}
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
            TreeCursor::default()
        }
    }

    fn goto(&mut self, mut c: TreeCursor) -> TreeNavResult {
        if let Some(mut element) = self.get_cur_element_mut() {
            if let ProductEditorElement::N{ t, editor, cur_depth } = element.deref_mut() {
                if let Some(e) = editor {
                    e.write().unwrap().goto(TreeCursor::default());
                }
                *cur_depth = self.get_cursor().tree_addr.len();
            }
        }

        if c.tree_addr.len() > 0 {
            self.cursor = Some(c.clone().tree_addr.remove(0));

            if let Some(mut element) = self.get_cur_element_mut() {
                if let ProductEditorElement::N{ t, editor, cur_depth } = element.deref_mut() {
                    if let Some(e) = editor {
                        e.write().unwrap().goto(c.clone());
                    }
                    *cur_depth = c.tree_addr.len() + 1;
                }
            }

            TreeNavResult::Continue
        } else {
            self.cursor = None;
            TreeNavResult::Exit
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        TreeNavResult::Exit
    }
/*
    fn goto_home(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let cur_mode = ce.get_cursor().leaf_mode;
                    let depth = ce.get_cursor().tree_addr.len();

                    if depth > 0 {
                        return match ce.goto_home() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *cur_depth = 0;

                                match self.pxev() {
                                    TreeNavResult::Exit => TreeNavResult::Exit,
                                    TreeNavResult::Continue => {
                                        for _x in 1..depth {
                                            self.dn();
                                            self.goto_end();
                                        }
                                        self.dn();
                                        self.set_leaf_mode(cur_mode);

                                        TreeNavResult::Continue
                                    }
                                }
                            },
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *cur_depth = 0;
                if c != 0 {
                    self.cursor = Some(0);
                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                        *cur_depth = self.get_cursor().tree_addr.len() + 1;
                    }
                    return TreeNavResult::Continue;
                }
            }
        }
        self.cursor = None;
        TreeNavResult::Exit
    }

    fn goto_end(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let cur_mode = ce.get_cursor().leaf_mode;
                    let depth = ce.get_cursor().tree_addr.len();

                    if depth > 0 {
                        match ce.goto_end() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *cur_depth = 0;

                                if c+1 < self.n_indices.len() {
                                    match self.nexd() {
                                        TreeNavResult::Exit => {
                                            return TreeNavResult::Exit
                                        },
                                        TreeNavResult::Continue => {
                                            for _x in 1..depth {
                                                self.dn();
                                            }

                                            self.dn();
                                            self.set_leaf_mode(cur_mode);
                                            self.goto_end();

                                            return TreeNavResult::Continue;
                                        }
                                    }
                                }
                            },
                            TreeNavResult::Continue => {return TreeNavResult::Continue; }
                        }
                    }
                }

                *cur_depth = 0;
                if c < self.n_indices.len()-1 {
                    self.cursor = Some(self.n_indices.len()-1);
                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                        *cur_depth = self.get_cursor().tree_addr.len();
                    }
                    return TreeNavResult::Continue;
                }
            }
        }
        self.cursor = None;
        TreeNavResult::Exit
    }

    fn pxev(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let depth = ce.get_cursor().tree_addr.len();
                    let cur_mode = ce.get_cursor().leaf_mode;

                    if depth > 0 {
                        return match ce.pxev() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *cur_depth = 0;

                                if c > 0 {
                                    self.cursor = Some(c-1);
                                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                                        *cur_depth = self.get_cursor().tree_addr.len();
                                    }

                                    for _x in 1..depth {
                                        self.dn();
                                        self.goto_end();
                                    }

                                    self.dn();
                                    self.set_leaf_mode(cur_mode);
                                    self.goto_end();

                                    TreeNavResult::Continue                                
                                } else {
                                    TreeNavResult::Exit
                                }
                            }
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *cur_depth = 0;
                if c > 0 {
                    self.cursor = Some(c-1);
                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                        *cur_depth = self.get_cursor().tree_addr.len();
                    }
                    return TreeNavResult::Continue;
                }
            }
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn nexd(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor.clone() {
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let depth = ce.get_cursor().tree_addr.len();
                    let cur_mode = ce.get_cursor().leaf_mode;

                    if depth > 0 {
                        return match ce.nexd() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *cur_depth = 0;

                                if c+1 < self.n_indices.len() {
                                    self.cursor = Some(c+1);
                                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                                        *cur_depth = self.get_cursor().tree_addr.len();
                                    }

                                    for _x in 1..depth {
                                        self.dn();
                                        self.goto_home();
                                    }

                                    self.dn();
                                    self.set_leaf_mode(cur_mode);

                                    TreeNavResult::Continue
                                } else {
                                    self.cursor = None;
                                    TreeNavResult::Exit
                                }
                            }
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *cur_depth = 0;
                if c+1 < self.n_indices.len() {
                    self.cursor = Some(c+1);
                    if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                        *cur_depth = self.get_cursor().tree_addr.len();
                    }

                    return TreeNavResult::Continue;
                }
            }
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn up(&mut self) -> TreeNavResult {
        if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
            if let Some(e) = editor {
                let mut ce = e.write().unwrap();
                *cur_depth = ce.get_cursor().tree_addr.len();
                if ce.get_cursor().tree_addr.len() > 0 {
                    ce.up();
                    return TreeNavResult::Continue;
                }
            }
            *cur_depth = 0;
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn dn(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    e.write().unwrap().dn();
                } else {
                    let e = make_editor(self.ctx.clone(), t, self.depth+1);
                    e.write().unwrap().dn();
                    *editor = Some(e);
                }
                *cur_depth = self.get_cursor().tree_addr.len();
            }
        } else {
            self.cursor = Some(0);
            if let Some(ProductEditorElement::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
                *cur_depth = self.get_cursor().tree_addr.len();
            }
        }

        TreeNavResult::Continue
}
    */
}

impl TerminalTreeEditor for ProductEditor {}

