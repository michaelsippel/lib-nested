use {
    crate::list::ListCursorMode,
    crate::tree::TreeCursor,
    crate::vec::VecBuffer,
    crate::core::{OuterViewPort},
    crate::sequence::{SequenceView, decorator::Separate},
    crate::singleton::{SingletonBuffer, SingletonView},
    cgmath::Vector2,
    crate::{
        terminal::{TerminalView, TerminalStyle, make_label}
        
    }
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



pub trait TreeNav {
    /* CORE
    */
    fn get_cursor(&self) -> TreeCursor {
        TreeCursor::default()
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        VecBuffer::<isize>::new().get_port().to_sequence()
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        SingletonBuffer::new(ListCursorMode::Select).get_port()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        TreeCursor::default()
    }

    fn get_max_depth(&self) -> usize {
        0
    }

    fn goby(&mut self, _direction: Vector2<isize>) -> TreeNavResult {
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

    fn get_cursor_widget(&self) -> OuterViewPort<dyn TerminalView> {
        VecBuffer::with_data(
            vec![
                make_label("@").with_fg_color((150, 80,230)),
                self.get_addr_view()
                    .map(|i|
                        make_label(&format!("{}", i)).with_fg_color((0, 100, 20)))
                    .separate(make_label(".").with_fg_color((150, 80,230)))
                    .to_grid_horizontal()
                    .flatten(),
                make_label(":").with_fg_color((150, 80,230)),
                self.get_mode_view()
                    .map(|mode| {
                        make_label(
                            match mode {
                                ListCursorMode::Insert => "INSERT",
                                ListCursorMode::Select => "SELECT"
                            })
                            .with_fg_color((200, 200, 20))
                    })
                    .to_grid()
                    .flatten(),
                make_label(":").with_fg_color((150, 80,230))
            ]
        ).get_port()
            .to_sequence()
            .to_grid_horizontal()
            .flatten()
    }
}

