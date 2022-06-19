use {
    crate::list::ListCursorMode,
    cgmath::Vector2
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TreeNavResult {
    Continue,
    Exit,
}

/*
impl From<TreeNavResult> for TerminalEditorResult {
    fn from(v: TreeNavResult) -> TerminalEditorResult {
        match v {
            TreeNavResult::Continue => TerminalEditorResult::Continue,
            TreeNavResult::Exit => TerminalEditorResult::Exit
        }
    }
}
 */

#[derive(Clone, Eq, PartialEq)]
pub struct TreeCursor {
    pub leaf_mode: ListCursorMode,
    pub tree_addr: Vec<isize>,
}

impl TreeCursor {
    pub fn home() -> Self {
        TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0]
        }
    }

    pub fn none() -> Self {
        TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![],
        }
    }
}

impl Default for TreeCursor {
    fn default() -> Self {
        TreeCursor::none()
    }
}

pub trait TreeNav {
    /* CORE
    */
    fn get_cursor(&self) -> TreeCursor {
        TreeCursor::default()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        TreeCursor::default()
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn goto(&mut self, _new_cursor: TreeCursor) -> TreeNavResult {
        TreeNavResult::Exit
    }

    /* HULL
    */
    fn set_addr(&mut self, addr: isize) -> TreeNavResult {
        let mut c = self.get_cursor();
        c.tree_addr[0] = addr;
        self.goto(c)
    }

    fn set_leaf_mode(&mut self, new_leaf_mode: ListCursorMode) -> TreeNavResult {
        let mut c = self.get_cursor();
        c.leaf_mode = new_leaf_mode;
        self.goto(c)
    }

    fn up(&mut self) -> TreeNavResult {
        self.goby(Vector2::new(0, -1))
    }

    fn dn(&mut self) -> TreeNavResult {
        self.goby(Vector2::new(0, 1))
    }

    fn pxev(&mut self) -> TreeNavResult {
        self.goby(Vector2::new(-1, 0))
    }

    fn nexd(&mut self) -> TreeNavResult {
        self.goby(Vector2::new(1, 0))
    }

    // TODO
    fn qpxev(&mut self) -> TreeNavResult {
        let mut c = self.get_cursor();
        match c.tree_addr.len() {
            0 => {
                self.goto(TreeCursor::home())
            },
            depth => {
                if c.tree_addr[depth-1] != 0 {
                    c.tree_addr[depth-1] = 0;
                } else {
                    for i in (0..depth-1).rev() {
                        if c.tree_addr[i] == 0 {
                            c.tree_addr[i] = -1;
                        } else {
                            c.tree_addr[i] -=1;
                            break;
                        }
                    }
                }

                self.goto(c)
            }
        }
    }

    fn qnexd(&mut self) -> TreeNavResult {
        let mut c = self.get_cursor_warp();
        match c.tree_addr.len() {
            0 => {
                TreeNavResult::Exit
            },
            depth => {
                if c.tree_addr[depth-1] != -1 {
                    c.tree_addr[depth-1] = -1;
                } else {
                    for i in (0..depth-1).rev() {
                        if c.tree_addr[i] == -1 {
                            c.tree_addr[i] = 0;
                        } else {
                            c.tree_addr[i] += 1;
                            break;
                        }
                    }
                }

                self.goto(c)
            }
        }
    }
}

use crate::terminal::{TerminalEditor};

pub trait TerminalTreeEditor : TerminalEditor + TreeNav + Send {}

