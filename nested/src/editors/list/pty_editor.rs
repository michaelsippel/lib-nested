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
            TerminalEditor, TerminalEvent,
            TerminalView,
            make_label
        },
        tree::{TreeCursor, TreeNav},
        diagnostics::{Diagnostics, make_error},
        tree::NestedNode,
        commander::Commander,
        PtySegment
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, Copy)]
pub enum ListStyle {
    Plain,
    HorizontalSexpr,
    VerticalSexpr,
    DoubleQuote,
    Tuple,
    EnumSet,
    Path,
    Hex
}

pub fn list_style_from_type(
    ctx: &Arc<RwLock<Context>>,
    typ: &TypeTerm
) -> Option<ListStyle> {
    let ctx = ctx.read().unwrap();

    match typ {
        TypeTerm::Type {
            id, args
        } => {
            if *id == ctx.get_typeid("List").unwrap() {
                Some(ListStyle::HorizontalSexpr)
            } else if *id == ctx.get_typeid("String").unwrap() {
                Some(ListStyle::DoubleQuote)
            } else if *id == ctx.get_typeid("Symbol").unwrap() {
                Some(ListStyle::Plain)
            } else if *id == ctx.get_typeid("PathSegment").unwrap() {
                Some(ListStyle::Plain)
            } else if *id == ctx.get_typeid("Path").unwrap() {
                Some(ListStyle::Path)
            } else if *id == ctx.get_typeid("PosInt").unwrap() {
                if args.len() > 0 {
                    match args[0] {
                        TypeTerm::Num(radix) => {
                            match radix {
                                16 => Some(ListStyle::Hex),
                                _ => Some(ListStyle::Plain)
                            }
                        }
                        _ => None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }

        _ => None
    }
}

impl ListStyle {
    fn get_split_char(&self) -> Option<char> {
        match self {
            ListStyle::Plain => None,
            ListStyle::DoubleQuote => None,
            ListStyle::HorizontalSexpr => Some(' '),
            ListStyle::VerticalSexpr => Some('\n'),
            ListStyle::Tuple => Some(','),
            ListStyle::EnumSet => Some(','),
            ListStyle::Path => Some('/'),
            ListStyle::Hex => None
        }
    }

    fn get_wrapper(&self) -> (&str, &str) {
        match self {
            ListStyle::Plain => ("", ""),
            ListStyle::HorizontalSexpr => ("(", ")"),
            ListStyle::VerticalSexpr => ("(", ")"),
            ListStyle::DoubleQuote => ("\"", "\""),
            ListStyle::Tuple => ("(", ")"),
            ListStyle::EnumSet => ("{", "}"),
            ListStyle::Path => ("<", ">"),
            ListStyle::Hex => ("0x", "")
        }
    }
}

pub struct PTYListEditor {
    pub editor: Arc<RwLock<ListEditor>>,
    style: ListStyle,
    depth: usize
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl PTYListEditor {
    pub fn new(
        ctx: Arc<RwLock<Context>>,
        typ: TypeTerm,
        style: ListStyle,
        depth: usize
    ) -> Self {
        Self::from_editor(
            Arc::new(RwLock::new(ListEditor::new(ctx, typ))), style, depth)
    }

    pub fn from_editor(
        editor: Arc<RwLock<ListEditor>>,
        style: ListStyle,
        depth: usize
    ) -> Self {
        PTYListEditor {
            style,
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

    pub fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
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
            .separate(make_label(&if let Some(c) = self.style.get_split_char() { format!("{}", c) } else { "".to_string() } ))
            .wrap(make_label(self.style.get_wrapper().0), make_label(self.style.get_wrapper().1))
            .to_grid_horizontal()
            .flatten()
    }

    pub fn into_node(self) -> NestedNode {
        let view = self.pty_view();        
        let editor = Arc::new(RwLock::new(self));

        let ed = editor.read().unwrap();
        let edd = ed.editor.read().unwrap();

   
        NestedNode::new()
            .set_data(edd.get_data())
            .set_cmd(editor.clone())
            .set_editor(ed.editor.clone())
            .set_nav(ed.editor.clone())
            .set_ctx(edd.ctx.clone())
            .set_view(view)
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

    pub fn split(e: &mut ListEditor, depth: usize) {
        let cur = e.get_cursor();
        if let Some(item) = e.get_item_mut() {
            if let Some(head_editor) = item.editor.clone() {

                let head = head_editor.downcast::<RwLock<ListEditor>>().unwrap();
                let mut head = head.write().unwrap();

                if cur.tree_addr.len() > 2 {
                    PTYListEditor::split(&mut head, depth+1);
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

                let style =
                    if let Some(item_type) = &item_type {
                        list_style_from_type(&tail.ctx, item_type)
                            .unwrap_or(
                                ListStyle::HorizontalSexpr
                            )
                    } else {
                        ListStyle::HorizontalSexpr
                    };

                let mut tail_node = PTYListEditor::from_editor(
                    Arc::new(RwLock::new(tail)),
                    style,
                    depth+1
                ).into_node();

                if let Some(item_type) = item_type {
                    tail_node.data = Some(ReprTree::ascend(
                        &tail_node.data.unwrap(),
                        item_type.clone()
                    ));
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

impl Commander for PTYListEditor {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        let mut e = self.editor.write().unwrap();

        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\t')))
                | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                    e.toggle_leaf_mode();
                    e.set_leaf_mode(ListCursorMode::Select);
                }
            _ => {
                let cur = e.cursor.get();
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
                                    let mut new_edit = Context::make_editor(&e.ctx, e.typ.clone(), self.depth).unwrap();
                                    new_edit.goto(TreeCursor::home());
                                    new_edit.handle_terminal_event(event);

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
/*
                                                if item_cur.tree_addr.len() > 1 {
                                                    let mut item = e.get_item_mut().unwrap();
                                                    item.handle_terminal_event(event);
                                            }
                                                */
                                            } else {
                                                item.handle_terminal_event(event);
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
/*
                                                if item_cur.tree_addr.len() > 1 {
                                                    let mut item = e.get_item_mut().unwrap();
                                                    item.handle_terminal_event(event);
                                            }
                                                */
                                            } else {
                                                item.handle_terminal_event(event);   
                                            }
                                        }

                                        TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                                            if Some(*c) == self.style.get_split_char() {
                                                PTYListEditor::split(&mut e, self.depth);
                                            } else {
                                                item.handle_terminal_event(event);
                                            }
                                        }
                                        _ => {
                                            item.handle_terminal_event(event);
                                        }
                                    }
                                } else {
                                    item.handle_terminal_event(event);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
