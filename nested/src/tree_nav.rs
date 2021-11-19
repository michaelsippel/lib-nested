use crate::list::ListCursorMode;

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
    pub tree_addr: Vec<usize>,
}

impl Default for TreeCursor {
    fn default() -> Self {
        TreeCursor {
            leaf_mode: ListCursorMode::Select,
            tree_addr: vec![],
        }
    }
}

pub trait TreeNav {
    fn up(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn dn(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn pxev(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn nexd(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn goto_home(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn goto_end(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn goto(&mut self, _new_cursor: TreeCursor) -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn get_cursor(&self) -> TreeCursor {
        TreeCursor::default()
    }
}

use crate::terminal::{TerminalEditor};

pub trait TerminalTreeEditor = TerminalEditor + TreeNav;
