use {
    r3vi::{
        view::{OuterViewPort, sequence::*},
        projection::decorate_sequence::*,
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::*,
        terminal::{TerminalEvent, TerminalView, make_label},
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::{Diagnostics},
        tree::NestedNode,
        PtySegment
    },
    std::sync::{Arc, RwLock},
    std::any::{Any},
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
            let mut ctx = node.ctx.as_ref().unwrap();
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

/*
    pub fn handle_node_event(&mut self, c: &NestedNode) -> TreeNavResult {
        
    }

    pub fn handle_char_event(&mut self, c: &char) -> TreeNavResult {
        
    }

    pub fn handle_term_event(&mut self, e: &TerminalEvent) -> TreeNavResult {
        
}
    */
}

use r3vi::view::singleton::SingletonView;
use crate::commander::ObjCommander;

impl ObjCommander for PTYListController {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        let cur = e.cursor.get();
        let cur_depth = e.get_cursor().tree_addr.len();

        let ctx = e.ctx.clone();
        let ctx = ctx.read().unwrap();

        let co = cmd_obj.read().unwrap();
        let cmd_type = co.get_type().clone();
        let term_event_type = ctx.type_term_from_str("( TerminalEvent )").unwrap();
        let nested_node_type = ctx.type_term_from_str("( NestedNode )").unwrap();
        let char_type = ctx.type_term_from_str("( Char )").unwrap();

        if cmd_type == nested_node_type {
            if let Some(node_view) = co.get_view::<dyn SingletonView<Item = NestedNode>>() {
                if let Some(idx) = cur.idx {
                    match cur.mode {
                        ListCursorMode::Select => {
                            *e.data.get_mut(idx as usize) = Arc::new(RwLock::new(node_view.get()));
                        }
                        ListCursorMode::Insert => {
                            e.data.insert(idx as usize, Arc::new(RwLock::new(node_view.get())));
                        }
                    }
                }
            }
        }

        if cmd_type == term_event_type {
            if let Some(te_view) = co.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                drop(co);
                let event = te_view.get();

                match event {
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                        | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                            e.toggle_leaf_mode();
                            e.set_leaf_mode(ListCursorMode::Select);

                            TreeNavResult::Continue
                        }
                    _ => {
                        if let Some(idx) = cur.idx {
                            match cur.mode {
                                ListCursorMode::Insert => {
                                    match event {
                                        TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                            e.delete_pxev();
                                            TreeNavResult::Continue
                                        }
                                        TerminalEvent::Input(Event::Key(Key::Delete)) => {
                                            e.delete_nexd();
                                            TreeNavResult::Continue
                                        }
                                        _ => {
                                            let mut node = Context::make_node(&e.ctx, e.typ.clone(), self.depth).unwrap();
                                            node.goto(TreeCursor::home());
                                            node.send_cmd_obj(cmd_obj);
/*
                                            if e.is_listlist() {
                                                if let Some(new_edit) = node.get_edit::<ListEditor>() {
                                                    if new_edit.data.len() == 0 {
                                                        remove = true;
                                                    }
                                                }
                                            }

                                            if ! remove {
                                            */
                                            e.insert(Arc::new(RwLock::new(node)));

                                            TreeNavResult::Continue
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
                                                        e.listlist_join_pxev(idx, &item);

                                                        /* Optional: recursive joining
                                                        
                                                        if item_cur.tree_addr.len() > 1 {
                                                        let mut item = e.get_item_mut().unwrap();
                                                        item.handle_terminal_event(event);
                                                        }
                                                         */
                                                        TreeNavResult::Continue
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj)
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
                                                        e.listlist_join_nexd(next_idx, &item);

                                                        /* Optional: recursive joining

                                                        if item_cur.tree_addr.len() > 1 {
                                                        let mut item = e.get_item_mut().unwrap();
                                                        item.handle_terminal_event(event);
                                                    }
                                                         */

                                                        TreeNavResult::Continue
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj)
                                                    }
                                                }

                                                TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                                                    if Some(c) == self.split_char {
                                                        e.listlist_split();
                                                        TreeNavResult::Continue
                                                    } else {
                                                        item.send_cmd_obj(cmd_obj)
                                                    }
                                                }
                                                _ => {
                                                    item.send_cmd_obj(cmd_obj)
                                                }
                                            }
                                        } else {
                                            item.send_cmd_obj(cmd_obj)
                                        }
                                    } else {
                                        TreeNavResult::Exit
                                    }
                                }
                            }
                        } else {
                            TreeNavResult::Exit
                        }
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        }

        else {
            drop(co);
            match cur.mode {
                ListCursorMode::Insert => {
                    let mut new_edit = Context::make_node(&e.ctx, e.typ.clone(), self.depth).unwrap();
                    new_edit.goto(TreeCursor::home());

                    match new_edit.send_cmd_obj(cmd_obj.clone()) {
                        TreeNavResult::Continue => {
                            e.insert(Arc::new(RwLock::new(new_edit)));
                            TreeNavResult::Continue
                        }

                        TreeNavResult::Exit => {
                            //eprintln!("listedit: exit from insert mode");
                            TreeNavResult::Exit
                        }
                    }
                },
                ListCursorMode::Select => {
                    if let Some(mut item) = e.get_item_mut() {

                        eprintln!("send cmd to child");
                        let mut i = item.write().unwrap();
                        let res = i.send_cmd_obj(cmd_obj.clone());

                        let close_char = i.close_char.get();
                        eprintln!("close char = {:?}", close_char);
                        drop(i);
                        drop(item);
                        
                        eprintln!("back");
                        match res {
                            TreeNavResult::Continue => {
                                TreeNavResult::Continue
                            }

                            TreeNavResult::Exit => {
                                if cmd_type == char_type {
                                    eprintln!("char event event");
                                    let co = cmd_obj.read().unwrap();
                                    if let Some(cmd_view) = co.get_view::<dyn SingletonView<Item = char>>() {
                                        drop(co);
                                        let c = cmd_view.get();

                                        if Some(c) == self.split_char {
                                            e.listlist_split();
                                            TreeNavResult::Continue
                                        } else if Some(c) == close_char {
                                            //eprintln!("listedit: exit from select (close)");
                                            //item.goto(TreeCursor::none());
                                            e.cursor.set(ListCursor {
                                                mode: ListCursorMode::Insert,
                                                idx: Some(cur.idx.unwrap_or(0)+1)
                                            });
                                            TreeNavResult::Continue
                                        } else {
                                            //eprintln!("listedit: exit from select mode");
                                            TreeNavResult::Exit
                                        }
                                    } else {
                                        TreeNavResult::Exit
                                    }
                                } else {
                                    TreeNavResult::Exit
                                }
                            }
                        }
                    } else {
                        TreeNavResult::Exit
                    }
                }
            }
        }
    }
}
