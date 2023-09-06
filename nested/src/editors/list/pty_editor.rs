use {
    r3vi::{
        view::{OuterViewPort, sequence::*},
        projection::decorate_sequence::*,
    },
    crate::{
        type_system::{Context, ReprTree},
        editors::list::*,
        terminal::{TerminalEvent, TerminalView, make_label},
        tree::{TreeCursor, TreeNav, TreeNavResult},
        tree::NestedNode,
        PtySegment
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTYListStyle {
    style: (String, String, String),
    depth: usize
}

impl PTYListStyle {
    pub fn new(style: (&str, &str, &str), depth: usize) -> PTYListStyle {
        PTYListStyle {
            style: (style.0.into(), style.1.into(), style.2.into()),
            depth
        }
    }

    pub fn get_seg_seq_view(&self, editor: &ListEditor) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let seg_seq = ListSegmentSequence::new(
            editor.get_cursor_port(),
            editor.get_data_port(),
            self.depth
        );
        let se = seg_seq.read().unwrap();
        se.get_view().map(move |segment| segment.pty_view())
    }

    pub fn pty_view(&self, editor: &ListEditor) -> OuterViewPort<dyn TerminalView> {
        let seg_seq = ListSegmentSequence::new(
            editor.get_cursor_port(),
            editor.get_data_port(),
            self.depth
        );
        let seg_seq = seg_seq.read().unwrap();

        seg_seq
            .get_view()
            .map(move |segment| segment.pty_view())
            .separate(make_label(&self.style.1))
            .wrap(make_label(&self.style.0), make_label(&self.style.2))
            .to_grid_horizontal()
            .flatten()
    }

    pub fn for_node(node: &mut NestedNode, style: (&str, &str, &str)) {
        node.view = Some(
            Self::new(style, node.depth.get())
                .pty_view(
                    &node.get_edit::<ListEditor>().unwrap().read().unwrap()
                )
        );
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTYListController {
    pub editor: Arc<RwLock<ListEditor>>,

    split_char: Option<char>,
    close_char: Option<char>,

    depth: usize
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl PTYListController {
    pub fn from_editor(
        editor: Arc<RwLock<ListEditor>>,
        split_char: Option<char>,
        close_char: Option<char>,
        depth: usize
    ) -> Self {
        PTYListController {
            editor,
            split_char,
            close_char,
            depth
        } 
    }

    pub fn for_node(
        node: &mut NestedNode,
        split_char: Option<char>,
        close_char: Option<char>
    ) {
        {
            let ctx = node.ctx.as_ref();
            let mut ctx = ctx.write().unwrap();

            if let Some(c) = split_char.as_ref() {
                ctx.meta_chars.push(*c);
            }
            if let Some(c) = close_char.as_ref() {
                ctx.meta_chars.push(*c);
            }
        }
        
        let editor = node.get_edit::<ListEditor>().unwrap();
        let controller = Arc::new(RwLock::new(PTYListController::from_editor( editor, split_char, close_char, node.depth.get() )));

        node.cmd.set(Some(controller.clone()));
        node.close_char.set(close_char);
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = NestedNode>> {
        self.editor.read().unwrap().get_data_port()
    }

    pub fn clear(&mut self) {
        self.editor.write().unwrap().clear();
    }

    pub fn get_item(&self) -> Option<NestedNode> {
        self.editor.read().unwrap().get_item()
    }

    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn handle_term_event(&mut self, event: &TerminalEvent, _cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        match event {
            TerminalEvent::Input(Event::Key(Key::Insert)) => {
                e.toggle_leaf_mode();
                TreeNavResult::Continue
            }
            _  => TreeNavResult::Continue
        }        
    }

    pub fn handle_meta_char(&mut self, c: char, child_close_char: Option<char>) -> TreeNavResult {
        eprintln!("handle meta char: got '{}', child_close={:?}, self.close={:?}, split={:?}", c, child_close_char, self.close_char, self.split_char);
        let mut e = self.editor.write().unwrap();
        let cur = e.cursor.get();
        
        if Some(c) == self.split_char
//            || Some(c) == child_close_char
        {
            e.listlist_split();
            TreeNavResult::Continue
        } else if Some(c) == child_close_char {
            e.goto(TreeCursor::none());
            e.cursor.set(ListCursor {
                mode: ListCursorMode::Insert,
                idx: Some(cur.idx.unwrap_or(0)+1)
            });
            TreeNavResult::Continue
        } else {
            TreeNavResult::Exit
        }
    }

    pub fn handle_any_event(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        let cur = e.cursor.get();
        let ctx = e.ctx.clone();
        let ctx = ctx.read().unwrap();

        match cur.mode {
            ListCursorMode::Insert => {
                let mut new_edit = Context::make_node(&e.ctx, e.typ.clone(), self.depth+1).unwrap();
                new_edit.goto(TreeCursor::home());

                match new_edit.send_cmd_obj(cmd_obj.clone()) {
                    TreeNavResult::Continue => {
                        e.insert(Arc::new(RwLock::new(new_edit)));
                        TreeNavResult::Continue
                    }
                    TreeNavResult::Exit => {
                        TreeNavResult::Exit
                    }
                }
            },
            ListCursorMode::Select => {
                if let Some(item) = e.get_item_mut() {
                    let res = item.write().unwrap().send_cmd_obj(cmd_obj.clone());
                    let child_close_char = item.read().unwrap().close_char.get();

                    match res {
                        TreeNavResult::Continue => TreeNavResult::Continue,
                        TreeNavResult::Exit => {
                            // child editor returned control, probably for meta-char handling..

                            if cmd_obj.read().unwrap().get_type().clone() == ctx.type_term_from_str("( Char )").unwrap() {
                                let co = cmd_obj.read().unwrap();
                                if let Some(cmd_view) = co.get_view::<dyn SingletonView<Item = char>>() {
                                    drop(co);
                                    drop(e);
                                    self.handle_meta_char(cmd_view.get(), child_close_char)
                                } else {
                                    TreeNavResult::Exit
                                }
                            } else {
                                TreeNavResult::Exit
                            }
                        }
                    }
                } else {
                    // cursor selects non existent item
                    TreeNavResult::Exit
                }
            }
        }
    }
}

use r3vi::view::singleton::SingletonView;
use crate::commander::ObjCommander;

impl ObjCommander for PTYListController {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        let cmd_type = cmd_obj.read().unwrap().get_type().clone();

        if cmd_type == (&e.ctx, "( ListCmd )").into()
        || cmd_type == (&e.ctx, "( NestedNode )").into()
        {
            e.send_cmd_obj( cmd_obj )
        }

        else if cmd_type == (&e.ctx, "( TerminalEvent )").into() {
            let co = cmd_obj.read().unwrap();
            if let Some(view) = co.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                drop( co );
                drop( e );
                self.handle_term_event( &view.get(), cmd_obj )
            } else {
                TreeNavResult::Exit
            }
        }

        else {
            drop( e );
            self.handle_any_event( cmd_obj )
        }
    }
}
