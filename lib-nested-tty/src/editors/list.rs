use {
    r3vi::{
        view::{ViewPort, OuterViewPort, sequence::*},
        projection::decorate_sequence::*,
    },
    nested::{
        repr_tree::{Context, ReprTree},
        editors::list::*,
        edit_tree::{TreeCursor, TreeNav, TreeNavResult, EditTree},
    },
    crate::{
        DisplaySegment,
        TerminalStyle,
        TerminalEvent, TerminalView, make_label,
        edit_tree::color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl DisplaySegment for ListSegment {
    fn display_view(&self) -> OuterViewPort<dyn TerminalView> {
        match self {
            ListSegment::InsertCursor => {
                make_label("|")
                    .map_item(move |_pt, atom| {
                     atom.add_style_front(TerminalStyle::fg_color((150,80,230)))
                        .add_style_front(TerminalStyle::bold(true))
                    })
            }
            ListSegment::Item{ editor, cur_dist } => {
                let e = editor.clone();
                let cur_dist = *cur_dist;
                editor.display_view().map_item(move |_pt, atom| {
                    let c = e.get_cursor();
                    let cur_depth = c.tree_addr.len();
                    let select =
                        if cur_dist == 0 {
                            cur_depth
                        } else {
                            usize::MAX
                        };
                    
                    atom
                        .add_style_back(bg_style_from_depth(select))
                        .add_style_back(TerminalStyle::bold(select==1))
                        .add_style_back(fg_style_from_depth(e.disp.depth.get_view().get()))
                })
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTYListStyle {
    style: (String, String, String)
}

impl PTYListStyle {
    pub fn new(style: (&str, &str, &str)) -> PTYListStyle {
        PTYListStyle {
            style: (style.0.into(), style.1.into(), style.2.into())
        }
    }

    pub fn get_seg_seq_view(&self, editor: &ListEditor) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let seg_seq = ListSegmentSequence::new(
            editor.get_cursor_port(),
            editor.get_data_port()
        );
        let se = seg_seq.read().unwrap();
        se.get_view().map(move |segment| segment.display_view())
    }

    pub fn pty_view(&self, editor: &ListEditor) -> OuterViewPort<dyn TerminalView> {
        let seg_seq = ListSegmentSequence::new(
            editor.get_cursor_port(),
            editor.get_data_port()
        );
        let seg_seq = seg_seq.read().unwrap();

        seg_seq
            .get_view()
            .map(move |segment| segment.display_view())
            .separate(make_label(&self.style.1))
            .wrap(make_label(&self.style.0), make_label(&self.style.2))
            .to_grid_horizontal()
            .flatten()
    }

    pub fn for_node(node: &mut EditTree, style: (&str, &str, &str)) {
        node.disp.view
            .write().unwrap()
            .insert_branch(ReprTree::new_leaf(
                Context::parse(&node.ctx, "TerminalView"),
                Self::new(style)
                    .pty_view(
                        &node.get_edit::<ListEditor>().unwrap().read().unwrap()
                ).into()
            ));
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

// todo: rename to CharController

pub struct PTYListController {
    pub editor: Arc<RwLock<ListEditor>>,

    split_char: Option<char>,
    close_char: Option<char>,

    depth: OuterViewPort<dyn SingletonView<Item = usize>>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl PTYListController {
    pub fn from_editor(
        editor: Arc<RwLock<ListEditor>>,
        split_char: Option<char>,
        close_char: Option<char>,
        depth: OuterViewPort<dyn SingletonView<Item = usize>>
    ) -> Self {
        PTYListController {
            editor,
            split_char,
            close_char,
            depth
        } 
    }

    pub fn for_node(
        node: &mut EditTree,
        split_char: Option<char>,
        close_char: Option<char>
    ) {
/*
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
*/
        let editor = node.get_edit::<ListEditor>().unwrap();
        let controller = Arc::new(RwLock::new(PTYListController::from_editor( editor, split_char, close_char, node.disp.depth.clone() )));

        node.ctrl.cmd.set(Some(controller.clone()));
        node.ctrl.close_char.set(close_char);
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = EditTree>> {
        self.editor.read().unwrap().get_data_port()
    }

    pub fn clear(&mut self) {
        self.editor.write().unwrap().clear();
    }

    pub fn get_item(&self) -> Option<EditTree> {
        self.editor.read().unwrap().get_item()
    }

    pub fn handle_term_event(&mut self, event: &TerminalEvent, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        match event {
            TerminalEvent::Input(Event::Key(Key::Insert)) => {
                e.toggle_leaf_mode();
                TreeNavResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                let ctx = e.ctx.clone();
                drop(e);
                self.handle_any_event(
                    ReprTree::from_char(&ctx, *c)
                )
            }
            _ => TreeNavResult::Continue
        }
    }

    pub fn handle_meta_char(&mut self, c: char, child_close_char: Option<char>) -> TreeNavResult {
//        eprintln!("handle meta char: got '{}', child_close={:?}, self.close={:?}, split={:?}", c, child_close_char, self.close_char, self.split_char);
        let mut e = self.editor.write().unwrap();
        let cur = e.cursor.get();

        if Some(c) == self.split_char
//            || Some(c) == child_close_char
        {
            e.listlist_split();
 //           eprintln!("done listlist split");
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
                let rt = ReprTree::new_arc(e.typ.clone());
                let new_edittree = ctx.setup_edittree(
                    rt,
                    self.depth.map(|d| d+1)
                );
                let mut ne = new_edittree.write().unwrap();
                match ne.send_cmd_obj(cmd_obj.clone()) {
                    TreeNavResult::Continue => {
                        drop(ne);
                        e.insert(new_edittree.clone());
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
                    let child_close_char = item.read().unwrap().ctrl.close_char.get();

                   match res {
                        TreeNavResult::Continue => TreeNavResult::Continue,
                        TreeNavResult::Exit => {
                            // child editor returned control, probably for meta-char handling..

                            if cmd_obj.read().unwrap().get_type().clone() == ctx.type_term_from_str("Char").unwrap() {
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
use nested::editors::ObjCommander;

impl ObjCommander for PTYListController {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let mut e = self.editor.write().unwrap();
        let cmd_type = cmd_obj.read().unwrap().get_type().clone();

        if cmd_type == Context::parse(&e.ctx, "ListCmd").into()
        || cmd_type == Context::parse(&e.ctx, "NestedNode").into()
        {
            e.send_cmd_obj( cmd_obj )
        }

        else if cmd_type == Context::parse(&e.ctx, "TerminalEvent").into() {
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
