use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{PTYListEditor},
        sequence::{SequenceView, SequenceViewExt, decorator::{PTYSeqDecorate, SeqDecorStyle}},
        singleton::{SingletonBuffer, SingletonView},
        vec::{VecBuffer},
        index::{buffer::IndexBuffer},
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView, make_label
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
        diagnostics::{Diagnostics, Message}
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::{Vector2, Point2}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct DigitEditor {
    radix: u32,
    data: SingletonBuffer<Option<char>>,
    msg: VecBuffer<Message>,
}

impl DigitEditor {
    pub fn new(radix: u32) -> Self {
        DigitEditor {
            radix,
            data: SingletonBuffer::new(None),
            msg: VecBuffer::new(),
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<u32>>> {
        let radix = self.radix;
        self.data.get_port().map(move |c| c?.to_digit(radix))
    }
}

impl TreeNav for DigitEditor {}
impl TerminalEditor for DigitEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        let radix = self.radix;
        self.data
            .get_port()
            .map(move |c| {
                TerminalAtom::new(
                    c.unwrap_or('?'),
                    if c.unwrap_or('?').to_digit(radix).is_some() {
                        TerminalStyle::fg_color((100, 140, 100))
                    } else {
                        //TerminalStyle::bg_color((90, 10, 10))
                        TerminalStyle::fg_color((200, 40, 40))
                    },
                )
            })
            .to_grid()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(' ')))
            | TerminalEvent::Input(Event::Key(Key::Char('\n'))) => TerminalEditorResult::Exit,
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));

                self.msg.clear();
                if c.to_digit(self.radix).is_none() {
                    let mut mb = IndexBuffer::new();
                    mb.insert_iter(vec![
                        (Point2::new(1, 0), make_label("invalid digit '")),
                        (Point2::new(2, 0), make_label(&format!("{}", *c))
                         .map_item(|p,a| a.add_style_back(TerminalStyle::fg_color((140,140,250))))),
                        (Point2::new(3, 0), make_label("'"))
                    ]);
                    self.msg.push(crate::diagnostics::make_error(mb.get_port().flatten()));
                }

                TerminalEditorResult::Exit
            }
            TerminalEvent::Input(Event::Key(Key::Backspace))
            | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                self.data.set(None);
                self.msg.clear();
                self.msg.push(crate::diagnostics::make_warn(make_label("empty digit")));
                TerminalEditorResult::Exit
            }
            _ => TerminalEditorResult::Continue,
        }
    }
}

impl TerminalTreeEditor for DigitEditor {}

impl Diagnostics for DigitEditor {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>> {
        self.msg.get_port().to_sequence()
    }
}

pub struct PosIntEditor {
    radix: u32,
    digits_editor: PTYListEditor<DigitEditor>
}

impl PosIntEditor {
    pub fn new(radix: u32) -> Self {
        PosIntEditor {
            radix,
            digits_editor: PTYListEditor::new(
                Box::new(move || Arc::new(RwLock::new(DigitEditor::new(radix)))) as Box<dyn Fn() -> Arc<RwLock<DigitEditor>> + Send + Sync>,
                SeqDecorStyle::Hex,
                ' ',
                0
            ),
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = u32>> {
        let radix = self.radix;
        self.digits_editor.editor
            .get_data_port()
            .filter_map(move |digit_editor| {
                digit_editor.read().unwrap().data.get()?.to_digit(radix)
            })
    }

    pub fn get_value(&self) -> u32 {
        let mut value = 0;
        let mut weight = 1;
        for digit_value in self
            .get_data_port()
            .get_view()
            .unwrap()
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            value += digit_value * weight;
            weight *= self.radix;
        }

        value
    }
}

impl Diagnostics for PosIntEditor {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>> {
        self.digits_editor.get_msg_port()
    }
}

impl TreeNav for PosIntEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.digits_editor.get_cursor()
    }
    fn get_cursor_warp(&self) -> TreeCursor {
        self.digits_editor.get_cursor_warp()
    }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        self.digits_editor.goto(cur)
    }
    fn goby(&mut self, cur: Vector2<isize>) -> TreeNavResult {
        self.digits_editor.goby(cur)
    }
}

impl TerminalEditor for PosIntEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        match self.radix {
            10 => {
                self.digits_editor.editor
                    .get_seg_seq_view()
                    .pty_decorate(SeqDecorStyle::Plain, 0)
            },
            16 => {
                self.digits_editor.editor
                    .get_seg_seq_view()
                    .pty_decorate(SeqDecorStyle::Hex, 0)
            }
            _ => {
                self.digits_editor.editor
                    .get_seg_seq_view()
                    .pty_decorate(SeqDecorStyle::Plain, 0)
            }
        }
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(' ')))
            | TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.digits_editor.up();
                TerminalEditorResult::Exit
            }

            event => self.digits_editor.handle_terminal_event(event),
        }
    }
}

impl TerminalTreeEditor for PosIntEditor {}

