use {
    r3vi::{
        view::{
            OuterViewPort,
            sequence::*,
        },
        projection::decorate_sequence::*,
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::{
            ListCursor, ListCursorMode,
            segment::{ListSegmentSequence},
            editor::ListEditor
        },
        terminal::{
            TerminalEvent,
            TerminalView,
            make_label
        },
        tree::{TreeCursor, TreeNav},
        diagnostics::{Diagnostics},
        tree::NestedNode,
        PtySegment
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTYListEditor {
    pub editor: Arc<RwLock<ListEditor>>,
    split_char: Option<char>,
    depth: usize
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl PTYListEditor {
    pub fn new(
        ctx: Arc<RwLock<Context>>,
        typ: TypeTerm,
        split_char: Option<char>,
        depth: usize
    ) -> Self {
        Self::from_editor(
            Arc::new(RwLock::new(ListEditor::new(ctx, typ))),
            split_char,
            depth
        )
    }

    pub fn from_editor(
        editor: Arc<RwLock<ListEditor>>,
        split_char: Option<char>,
        depth: usize
    ) -> Self {
        PTYListEditor {
            split_char,
            depth,
            editor,
        } 
    }

    pub fn get_seg_seq_view(&self) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let seg_seq = ListSegmentSequence::new(
            self.editor.read().unwrap().get_cursor_port(),
            self.editor.read().unwrap().get_data_port(),
            self.depth
        );
        let se = seg_seq.read().unwrap();
        se.get_view().map(move |segment| segment.pty_view())
    }

    pub fn pty_view(
        &self,
        display_style: (&str, &str, &str),
    ) -> OuterViewPort<dyn TerminalView> {
        let editor = self.editor.read().unwrap();

        let seg_seq = ListSegmentSequence::new(
            editor.get_cursor_port(),
            editor.get_data_port(),
            self.depth
        );
        let seg_seq = seg_seq.read().unwrap();

        seg_seq
            .get_view()
            .map(move |segment| segment.pty_view())
            .separate(make_label(display_style.1))
            .wrap(make_label(display_style.0), make_label(display_style.2))
            .to_grid_horizontal()
            .flatten()
    }
    
    pub fn into_node(self) -> NestedNode {
        let depth = self.depth;
        let editor = Arc::new(RwLock::new(self));

        let ed = editor.read().unwrap();
        let edd = ed.editor.read().unwrap();
   
        NestedNode::new(depth)
            .set_data(edd.get_data())
            .set_cmd(editor.clone())
            .set_editor(ed.editor.clone())
            .set_nav(ed.editor.clone())
            .set_ctx(edd.ctx.clone())
            .set_diag(
                edd.get_data_port()
                    .enumerate()
                    .map(
                        |(idx, item_editor)| {
                            let idx = *idx;
                            item_editor
                                .get_msg_port()
                                .map(
                                    move |msg| {
                                        let mut msg = msg.clone();
                                        msg.addr.insert(0, idx);
                                        msg
                                    }
                                )
                        }
                    )
                .flatten()
            )
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

    pub fn split(e: &mut ListEditor) {
        let cur = e.get_cursor();
        if let Some(item) = e.get_item_mut() {
            let depth = item.depth;

            if let Some(head_editor) = item.editor.clone() {

                let head = head_editor.downcast::<RwLock<ListEditor>>().unwrap();
                let mut head = head.write().unwrap();

                if cur.tree_addr.len() > 2 {
                    PTYListEditor::split(&mut head);
                }

                let mut tail = head.split();

                head.goto(TreeCursor::none());

                tail.cursor.set(
                    ListCursor {
                        idx: Some(0),
                        mode: if cur.tree_addr.len() > 2 {
                            ListCursorMode::Select
                        } else {
                            ListCursorMode::Insert
                        }
                    }
                );

                let item_type =
                    if let Some(data) = item.data.clone() {
                        let data = data.read().unwrap();
                        Some(data.get_type().clone())
                    } else {
                        None
                    };

                let mut tail_node = tail.into_node(depth);
                tail_node = tail_node.set_ctx(item.ctx.clone().unwrap());

                if let Some(item_type) = item_type {
                    tail_node = tail_node.morph(item_type);
                }
                
                e.insert(
                    tail_node
                );
            }
        }
    }

    fn join_pxev(e: &mut ListEditor, idx: isize, item: &NestedNode) {
        {
            let prev_editor = e.data.get_mut(idx as usize-1);
            let prev_editor = prev_editor.editor.clone();
            let prev_editor = prev_editor.unwrap().downcast::<RwLock<ListEditor>>().unwrap();
            let mut prev_editor = prev_editor.write().unwrap();

            let cur_editor = item.editor.clone().unwrap();
            let cur_editor = cur_editor.downcast::<RwLock<ListEditor>>().unwrap();
            let cur_editor = cur_editor.write().unwrap();

            prev_editor.join(&cur_editor);
        }

        e.cursor.set(
            ListCursor {
                idx: Some(idx - 1), mode: ListCursorMode::Select
            }
        );

        e.data.remove(idx as usize);
    }

    fn join_nexd(e: &mut ListEditor, next_idx: usize, item: &NestedNode) {
        {
            let next_editor = e.data.get_mut(next_idx).editor.clone();
            let next_editor = next_editor.unwrap().downcast::<RwLock<ListEditor>>().unwrap();
            let next_editor = next_editor.write().unwrap();

            let cur_editor = item.editor.clone().unwrap();
            let cur_editor = cur_editor.downcast::<RwLock<ListEditor>>().unwrap();
            let mut cur_editor = cur_editor.write().unwrap();

            cur_editor.join(&next_editor);
        }
        e.data.remove(next_idx);
    }
}

use r3vi::view::singleton::SingletonView;
use crate::commander::ObjCommander;

impl ObjCommander for PTYListEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        let mut e = self.editor.write().unwrap();
        let cur = e.cursor.get();

        let ctx = e.ctx.clone();
        let ctx = ctx.read().unwrap();

        let co = cmd_obj.read().unwrap();
        let cmd_type = co.get_type().clone();
        let term_event_type = ctx.type_term_from_str("( TerminalEvent )").unwrap();
        let char_type = ctx.type_term_from_str("( Char )").unwrap();

        if cmd_type == term_event_type {
            if let Some(te_view) = co.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                drop(co);
                let event = te_view.get();

                match event {
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                        | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                            e.toggle_leaf_mode();
                            e.set_leaf_mode(ListCursorMode::Select);
                        }
                    _ => {
                        if let Some(idx) = cur.idx {
                            match cur.mode {
                                ListCursorMode::Insert => {
                                    match event {
                                        TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                            e.delete_pxev();
                                        }
                                        TerminalEvent::Input(Event::Key(Key::Delete)) => {
                                            e.delete_nexd();
                                        }
                                        _ => {
                                            let mut new_edit = Context::make_node(&e.ctx, e.typ.clone(), self.depth).unwrap();
                                            new_edit.goto(TreeCursor::home());
                                            new_edit.send_cmd_obj(cmd_obj);

                                            e.insert(new_edit);
                                        }
                                    }
                                },
                                ListCursorMode::Select => {
                                    if let Some(mut item) = e.get_item().clone() {
                                        if e.is_listlist() {
                                            match event {
                                                TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                                    let item_cur = item.get_cursor();

                                                    if idx > 0
                                                        && item_cur.tree_addr.iter().fold(
                                                            true,
                                                            |is_zero, x| is_zero && (*x == 0)
                                                        )
                                                    {
                                                        PTYListEditor::join_pxev(&mut e, idx, &item);

                                                        /* Optional: recursive joining
                                                        
                                                        if item_cur.tree_addr.len() > 1 {
                                                        let mut item = e.get_item_mut().unwrap();
                                                        item.handle_terminal_event(event);
                                                    }
                                                         */
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj);
                                                    }
                                                }
                                                TerminalEvent::Input(Event::Key(Key::Delete)) => {
                                                    let item_cur = item.get_cursor_warp();
                                                    let next_idx = idx as usize + 1;

                                                    if next_idx < e.data.len()
                                                        && item_cur.tree_addr.iter().fold(
                                                            true,
                                                            |is_end, x| is_end && (*x == -1)
                                                        )
                                                    {
                                                        PTYListEditor::join_nexd(&mut e, next_idx, &item);

                                                        /* Optional: recursive joining

                                                        if item_cur.tree_addr.len() > 1 {
                                                        let mut item = e.get_item_mut().unwrap();
                                                        item.handle_terminal_event(event);
                                                    }
                                                         */
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj);
                                                    }
                                                }

                                                TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                                                    if Some(c) == self.split_char {
                                                        PTYListEditor::split(&mut e);
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj);
                                                    }
                                                }
                                                _ => {
                                                    item.send_cmd_obj(cmd_obj);
                                                }
                                            }
                                        } else {
                                            item.send_cmd_obj(cmd_obj);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }       
            }
        } else if cmd_type == char_type && cur.mode == ListCursorMode::Select {
            if let Some(cmd_view) = co.get_view::<dyn SingletonView<Item = char>>() {
                drop(co);
                let c = cmd_view.get();

                if Some(c) == self.split_char {
                    PTYListEditor::split(&mut e);
                } else {
                    if let Some(mut item) = e.get_item_mut() {
                        item.send_cmd_obj(cmd_obj);
                    }
                }
            }
        } else {
            drop(co);

            match cur.mode {
                ListCursorMode::Insert => {
                    let mut new_edit = Context::make_node(&e.ctx, e.typ.clone(), self.depth).unwrap();
                    new_edit.goto(TreeCursor::home());
                    new_edit.send_cmd_obj(cmd_obj);

                    e.insert(new_edit);                    
                },
                ListCursorMode::Select => {
                    if let Some(mut item) = e.get_item_mut() {
                        item.send_cmd_obj(cmd_obj);
                    }                    
                }
            }
        }
    }
}
