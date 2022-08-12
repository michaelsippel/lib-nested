use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{ListEditor},
        sequence::{SequenceView, SequenceViewExt},
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::Vector2
};

pub struct CharEditor {
    data: SingletonBuffer<Option<char>>
}

impl CharEditor {
    pub fn new() -> Self {
        CharEditor {
            data: SingletonBuffer::new(None)
        }
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<char>>> {
        self.data.get_port()
    }

    pub fn get(&self) -> char {
        self.get_port().get_view().unwrap().get().unwrap_or('?')
    }
}

impl TreeNav for CharEditor {}
impl TerminalEditor for CharEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.data
            .get_port()
            .map(move |c| {                
                TerminalAtom::new(
                    c.unwrap_or('?'),
                    TerminalStyle::fg_color((100, 140, 100)),
                )
            })
            .to_grid()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => TerminalEditorResult::Exit,
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));
                TerminalEditorResult::Exit
            }
            TerminalEvent::Input(Event::Key(Key::Backspace))
            | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                self.data.set(None);
                TerminalEditorResult::Exit
            }
            _ => TerminalEditorResult::Exit,
        }
    }
}

impl TerminalTreeEditor for CharEditor {}


