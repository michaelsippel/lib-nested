use {
    std::sync::RwLock,
    termion::event::{Key, Event},
    crate::{
        core::{ViewPort, OuterViewPort},
        singleton::{SingletonView, SingletonBuffer},
        vec::VecBuffer,
        terminal::{TerminalView, TerminalStyle, TerminalEvent, TerminalEditor, TerminalEditorResult},
        tree_nav::{TreeNav, TreeNavResult}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct CharEditor {
    data: SingletonBuffer<Option<char>>,
    data_port: ViewPort<dyn SingletonView<Item = Option<char>>>
}

impl CharEditor {
    pub fn new() -> Self {
        let mut data_port = ViewPort::new();
        CharEditor {
            data: SingletonBuffer::new(None, data_port.inner()),
            data_port
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<char>>> {
        self.data_port.outer()
    }
}

impl TreeNav for CharEditor {}
impl TerminalEditor for CharEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        crate::terminal::make_label(
            &if let Some(c) = self.data.get() {
                c.to_string()
            } else {
                "".to_string()
            })
            .map_item(
                |_idx, atom|
                atom.add_style_back(TerminalStyle::fg_color((120, 200, 10)))
        )
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

/*
pub struct ArgListEditor {
    
}

impl TreeNav for ArgListEditor {
    
}

impl TerminalEditor for ArgListEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(' '))) => {
                // list.get_arg()
                // split
            }
            _ => {
                
            }
        }
    }
}
*/
