use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{sexpr::ListDecoration, ListEditor},
        sequence::SequenceView,
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView,
        },
        tree_nav::{TreeCursor, TreeNav, TreeNavResult},
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct CharEditor {
    data: SingletonBuffer<Option<char>>,
    data_port: ViewPort<dyn SingletonView<Item = Option<char>>>,
}

impl CharEditor {
    pub fn new() -> Self {
        let data_port = ViewPort::new();
        CharEditor {
            data: SingletonBuffer::new(None, data_port.inner()),
            data_port,
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<char>>> {
        self.data_port.outer()
    }
}

impl TreeNav for CharEditor {}
impl TerminalEditor for CharEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        crate::terminal::make_label(&if let Some(c) = self.data.get() {
            c.to_string()
        } else {
            "".to_string()
        })
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => TerminalEditorResult::Continue,
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));
                TerminalEditorResult::Exit
            }
            TerminalEvent::Input(Event::Key(Key::Backspace))
            | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                self.data.set(None);
                TerminalEditorResult::Exit
            }
            _ => TerminalEditorResult::Continue,
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct StringEditor {
    chars_editor:
        ListEditor<CharEditor, Box<dyn Fn() -> Arc<RwLock<CharEditor>> + Send + Sync + 'static>>,
}

impl StringEditor {
    pub fn new() -> Self {
        StringEditor {
            chars_editor: ListEditor::new(
                Box::new(move || Arc::new(RwLock::new(CharEditor::new()))),
                crate::list::ListEditorStyle::String,
            ),
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = char>> {
        self.chars_editor
            .get_data_port()
            .map(|char_editor| char_editor.read().unwrap().data.get().unwrap())
    }
}

impl TreeNav for StringEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.chars_editor.get_cursor()
    }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        self.chars_editor.goto(cur)
    }
    fn goto_home(&mut self) -> TreeNavResult {
        self.chars_editor.goto_home()
    }
    fn goto_end(&mut self) -> TreeNavResult {
        self.chars_editor.goto_end()
    }
    fn pxev(&mut self) -> TreeNavResult {
        self.chars_editor.pxev()
    }
    fn nexd(&mut self) -> TreeNavResult {
        self.chars_editor.nexd()
    }
    fn up(&mut self) -> TreeNavResult {
        self.chars_editor.up()
    }
    fn dn(&mut self) -> TreeNavResult {
        TreeNavResult::Exit
    }
}

impl TerminalEditor for StringEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.chars_editor
            .get_seg_seq_view()
            .decorate("\"", "\"", "", 1)
            .to_grid_horizontal()
            .flatten()
            .map_item(|_idx, atom| atom.add_style_back(TerminalStyle::fg_color((120, 200, 10))))
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.chars_editor.up();
                TerminalEditorResult::Exit
            }
            event => self.chars_editor.handle_terminal_event(event),
        }
    }
}
