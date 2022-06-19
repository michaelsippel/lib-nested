use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{
            ListCursor, ListCursorMode,
            ListSegment, ListSegmentSequence,
            ListEditor
        },
        sequence::{SequenceView, decorator::{SeqDecorStyle, PTYSeqDecorate}},
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            make_label, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
        vec::VecBuffer,
        color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    cgmath::Vector2
};

pub struct PTYListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    pub editor: ListEditor<ItemEditor>,

    style: SeqDecorStyle,
    depth: usize,

    port: ViewPort<dyn TerminalView>
}

impl<ItemEditor> PTYListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    pub fn new(
        make_item_editor: Box<dyn Fn() -> Arc<RwLock<ItemEditor>> + Send + Sync>,
        style: SeqDecorStyle,
        depth: usize
    ) -> Self {
        let port = ViewPort::new();
        PTYListEditor {
            editor: ListEditor::new(make_item_editor, depth),
            style,
            depth,
            port
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>> {
        self.editor.get_data_port()
    }
    
    pub fn clear(&mut self) {
        self.editor.clear();
    }
    
    pub fn get_item(&self) -> Option<Arc<RwLock<ItemEditor>>> {
        self.editor.get_item()
    }
    
    pub fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    pub fn set_style(&mut self, style: SeqDecorStyle) {
        self.style = style;
    }
}

impl<ItemEditor> TerminalEditor for PTYListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor
            .get_seg_seq_view()
            .pty_decorate(self.style, self.depth)
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        let mut cur = self.editor.cursor.get();
        if let Some(idx) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => match event {
                    TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                        if idx > 0 && idx <= self.editor.data.len() as isize {
                            cur.idx = Some(idx as isize - 1);
                            self.editor.cursor.set(cur);
                            self.editor.data.remove(idx as usize - 1);

                            if self.editor.data.len() > 0 {
                                TerminalEditorResult::Continue
                            } else {
                                TerminalEditorResult::Exit
                            }
                        } else {
                            TerminalEditorResult::Exit
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Delete)) => {
                        if idx < self.editor.data.len() as isize {
                            self.editor.data.remove(idx as usize);
                            TerminalEditorResult::Continue
                        } else {
                            TerminalEditorResult::Exit
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                    | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                        self.editor.set_leaf_mode(ListCursorMode::Select);
                        TerminalEditorResult::Continue
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        self.editor.goto(TreeCursor::none());
                        TerminalEditorResult::Exit
                    }
                    _ => {
                        let new_edit = (self.editor.make_item_editor)();
                        self.editor.data.insert(idx as usize, new_edit.clone());
                        self.editor.set_leaf_mode(ListCursorMode::Select);

                        let mut ne = new_edit.write().unwrap();
                        ne.goto(TreeCursor::home());

                        match ne.handle_terminal_event(event) {
                            TerminalEditorResult::Exit => {
                                self.editor.cursor.set(ListCursor {
                                    mode: ListCursorMode::Insert,
                                    idx: Some(idx as isize + 1),
                                });
                            }
                            _ => {}
                        }
                        TerminalEditorResult::Continue
                    }
                },
                ListCursorMode::Select => {
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Char('\t')))
                            | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                                self.editor.set_leaf_mode(ListCursorMode::Insert);
                                TerminalEditorResult::Continue
                            }
                        ev => {
                            if let Some(e) = self.editor.get_item() {
                                match e.write().unwrap().handle_terminal_event(ev) {
                                    TerminalEditorResult::Exit => {

                                        match ev {
                                            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                                self.editor.data.remove(idx as usize);
                                                self.editor.cursor.set(ListCursor {
                                                    mode: ListCursorMode::Insert,
                                                    idx: Some(idx as isize),
                                                });                             
                                            }
                                            _ => {
                                                self.editor.cursor.set(ListCursor {
                                                    mode: ListCursorMode::Insert,
                                                    idx: Some(idx as isize + 1),
                                                });                                                
                                            }
                                        }
                                    }
                                    TerminalEditorResult::Continue => {
                                        
                                    }
                                }
                            }

                            TerminalEditorResult::Continue
                        }
                    }
                }
            }
        } else {
            TerminalEditorResult::Continue
        }
    }
}

impl<ItemEditor> TreeNav for PTYListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    fn get_cursor_warp(&self) -> TreeCursor {
        self.editor.get_cursor_warp()
    }

    fn get_cursor(&self) -> TreeCursor {
        self.editor.get_cursor()
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        self.editor.goby(direction)
    }

    fn goto(&mut self, cursor: TreeCursor) -> TreeNavResult {
        self.editor.goto(cursor)
    }
}


impl<ItemEditor> TerminalTreeEditor for PTYListEditor<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{}

