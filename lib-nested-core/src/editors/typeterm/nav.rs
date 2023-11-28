use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
            sequence::*
        }
    },
    crate::{
        edit_tree::{TreeNav, TreeCursor, TreeNavResult, TreeHeightOp},
        editors::{typeterm::TypeTermEditor, list::ListCursorMode}
    },
    cgmath::Vector2
};

impl TreeNav for TypeTermEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.cur_node.get().get_cursor()
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        self.cur_node.get_port().map(|x| x.get_addr_view()).to_sequence().flatten()   
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        self.cur_node.get_port().map(|x| x.get_mode_view()).flatten()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        self.cur_node.get().get_cursor_warp()
    }

    fn get_height(&self, op: &TreeHeightOp) -> usize {
        self.cur_node.get().get_height(op)
    }

    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        self.cur_node.get_mut().goby(dir)
    }

    fn goto(&mut self, new_cur: TreeCursor) -> TreeNavResult {
        self.cur_node.get_mut().goto(new_cur)
    }
}
