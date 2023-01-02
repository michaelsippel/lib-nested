use {
    crate::{
        core::{OuterViewPort},
        type_system::{Context, TypeTerm},
        list::{
            ListCursor, ListCursorMode,
            ListEditor
        },
        sequence::{SequenceView, decorator::{SeqDecorStyle, PTYSeqDecorate}},
        terminal::{
            TerminalEditor, TerminalEditorResult, TerminalEvent,
            TerminalView,
        },
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::{Diagnostics},
        tree::NestedNode, Nested,
        commander::Commander
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    cgmath::Vector2
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTYListEditor {
    pub editor: Arc<RwLock<ListEditor>>,
    split_char: Option<char>,
 
    style: SeqDecorStyle,
    depth: usize,

    pub diag: OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>>,
    pub view: OuterViewPort<dyn TerminalView>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl PTYListEditor {
    pub fn new(
        ctx: Arc<RwLock<Context>>,
        typ: TypeTerm,
        style: SeqDecorStyle,
        split_char: Option<char>,
        depth: usize
    ) -> Self {
        Self::from_editor(
            ListEditor::new(ctx, typ, depth), style, split_char, depth)
    }

    pub fn from_editor(
        editor: ListEditor,
        style: SeqDecorStyle,
        split_char: Option<char>,
        depth: usize
    ) -> Self {
        PTYListEditor {
            split_char,
            style,
            depth,

            view: editor.get_seg_seq_view().pty_decorate(style, depth),
            diag: editor.get_data_port()
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
                    .flatten(),

            editor: Arc::new(RwLock::new(editor)),
        } 
    }

    pub fn into_node(self) -> NestedNode {
        let editor = Arc::new(RwLock::new(self));

        let ed = editor.read().unwrap();
        let edd = ed.editor.read().unwrap();

        NestedNode::new()
            .set_cmd(editor.clone())
            .set_nav(ed.editor.clone())
            .set_ctx(edd.ctx.clone())
            .set_diag(ed.diag.clone())
            .set_view(ed.view.clone())
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

    pub fn set_style(&mut self, style: SeqDecorStyle) {
        self.style = style;
    }
}

impl Commander for PTYListEditor {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        let mut e = self.editor.write().unwrap();

        let mut cur = e.cursor.get();
        if let Some(idx) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => match event {
                    TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                        if idx > 0 && idx <= e.data.len() as isize {
                            cur.idx = Some(idx as isize - 1);
                            e.cursor.set(cur);
                            e.data.remove(idx as usize - 1);
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Delete)) => {
                        if idx < e.data.len() as isize {
                            e.data.remove(idx as usize);
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                    | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                        e.set_leaf_mode(ListCursorMode::Select);
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        e.goto(TreeCursor::none());
                    }
                    _ => {
                        let mut new_edit = Context::make_editor(&e.ctx, e.typ.clone(), self.depth+1).unwrap();
                        e.data.insert(idx as usize, new_edit.clone());
                        e.set_leaf_mode(ListCursorMode::Select);

                        new_edit.goto(TreeCursor::home());
                        new_edit.handle_terminal_event(event);

                        if self.split_char.is_none() {
                            e.cursor.set(ListCursor {
                                mode: ListCursorMode::Insert,
                                idx: Some(idx as isize + 1),
                            });
                        }
                    }
                },
                ListCursorMode::Select => {
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Char('\t')))
                            | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                                e.set_leaf_mode(ListCursorMode::Insert);
                            }

                        TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                            if Some(*c) == self.split_char {
                                let c = e.cursor.get();
                                e.goto(TreeCursor::none());
                                e.cursor.set(ListCursor {
                                    mode: ListCursorMode::Insert,
                                    idx: Some(1 + c.idx.unwrap_or(0))
                                });
                            } else {
                                if let Some(mut ce) = e.get_item_mut() {
                                    ce.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char(*c))));
                                    //match 
                                    if self.split_char.is_none() {
                                    //    TerminalEditorResult::Exit =>
                                        {
                                            e.cursor.set(ListCursor {
                                                mode: ListCursorMode::Insert,
                                                idx: Some(idx as isize + 1),
                                            });
                                        }
                                      //  TerminalEditorResult::Continue => {
                                      //  }
                                    }
                                }
                            }
                        }
                        ev => {
                            if let Some(mut ce) = e.get_item_mut() {
                                ce.handle_terminal_event(ev);
/*
                                TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                                e.data.remove(idx as usize);
                                                e.cursor.set(ListCursor {
                                                    mode: ListCursorMode::Insert,
                                                    idx: Some(idx as isize),
                                                });                             
                                            }
                                            _ => {
                                                e.cursor.set(ListCursor {
                                                    mode: ListCursorMode::Insert,
                                                    idx: Some(idx as isize + 1),
                                                });                                                
                                            }
                                        }
                                    }
                                    TerminalEditorResult::Continue => {
                                        
                                    }
                            }
                                */
                            }
                        }
                    }
                }
            }
        }
    }
}

