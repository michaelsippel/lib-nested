use {
    std::sync::RwLock,
    termion::event::{Key, Event},
    crate::{
        core::{ViewPort, OuterViewPort},
        singleton::{SingletonView, SingletonBuffer},
        vec::VecBuffer,
        terminal::{TerminalAtom, TerminalStyle, TerminalView, TerminalEvent, TerminalEditor, TerminalEditorResult},
        tree_nav::{TreeNav, TreeNavResult}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct DigitEditor {
    radix: u32,
    data: SingletonBuffer<Option<char>>,
    data_port: ViewPort<dyn SingletonView<Item = Option<char>>>
}

impl DigitEditor {
    pub fn new(radix: u32) -> Self {
        let mut data_port = ViewPort::new();
        DigitEditor {
            radix,
            data: SingletonBuffer::new(None, data_port.inner()),
            data_port
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<u32>>> {
        let radix = self.radix;
        self.data_port.outer().map(
            move |c| c.unwrap_or('?').to_digit(radix)
        )
    }
}

impl TreeNav for DigitEditor {}
impl TerminalEditor for DigitEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        let radix = self.radix;
        self.data_port.outer().map(
            move |c| TerminalAtom::new(
                c.unwrap_or('?'),
                if c.unwrap_or('?').to_digit(radix).is_some() {
                    TerminalStyle::fg_color((100, 140, 100))
                } else {
                    TerminalStyle::fg_color((200, 0, 0))
                }
            )
        ).to_grid()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) =>
                TerminalEditorResult::Continue,
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));
                TerminalEditorResult::Exit
            }
            TerminalEvent::Input(Event::Key(Key::Backspace)) |
            TerminalEvent::Input(Event::Key(Key::Delete)) => {
                self.data.set(None);
                TerminalEditorResult::Exit
            }
            _ => TerminalEditorResult::Continue
        }
    }
}

struct PosIntEditor {
    
}

