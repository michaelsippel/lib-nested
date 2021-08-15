
#[derive(Eq, PartialEq)]
pub enum TreeNavResult {
    Continue,
    Exit
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
    
    fn goto(&mut self, tree_addr: Vec<usize>)  -> TreeNavResult {
        TreeNavResult::Exit
    }
}

use crate::terminal::{TerminalView, TerminalEditor};

pub trait TerminalTreeEditor = TerminalEditor + TreeNav;

