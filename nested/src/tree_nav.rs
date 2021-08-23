
use crate::list::ListCursorMode;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TreeNavResult {
    Continue,
    Exit
}

#[derive(Clone, Eq, PartialEq)]
pub struct TreeCursor {
    pub leaf_mode: ListCursorMode,
    pub tree_addr: Vec<usize>
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

    fn goto(&mut self, new_cursor: Option<TreeCursor>)  -> TreeNavResult {
        TreeNavResult::Exit
    }

    fn get_cursor(&self) -> Option<TreeCursor> {
        None
    }
}

use crate::terminal::{TerminalView, TerminalEditor};

pub trait TerminalTreeEditor = TerminalEditor + TreeNav;

