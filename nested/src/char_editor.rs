use {
    crate::{
        core::{OuterViewPort, ViewPort, Context},
        list::{ListEditor},
        sequence::{SequenceView, SequenceViewExt},
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::Diagnostics,
        Nested, tree::NestedNode, Commander
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::Vector2
};

pub struct CharEditor {
    data: SingletonBuffer<Option<char>>
}

impl Commander for CharEditor {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));
            }
            TerminalEvent::Input(Event::Key(Key::Backspace))
            | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                self.data.set(None);
            }
            _ => {}
        }
    }
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

    pub fn new_node(ctx: &Arc<RwLock<Context>>) -> NestedNode {
        let data = SingletonBuffer::new(None);

        NestedNode::new()
            .set_ctx(ctx.clone())
            .set_view(data
                      .get_port()
                      .map(move |c| {
                          match c {
                              Some(c) => TerminalAtom::from(c),
                              None => TerminalAtom::new('*', TerminalStyle::fg_color((255,0,0)))
                          }
                      })
                      .to_grid()
            )
            .with_cmd(
                Arc::new(RwLock::new(CharEditor{ data }))
            )
    }
}

use crate::StringGen;
impl StringGen for CharEditor {
    fn get_string(&self)  -> String {
        String::from(self.get())
    }
}

