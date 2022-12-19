use {
    crate::{
        core::{OuterViewPort, Context, TypeTerm},
        list::{PTYListEditor},
        sequence::{SequenceView, SequenceViewExt, decorator::{PTYSeqDecorate, SeqDecorStyle}},
        singleton::{SingletonBuffer, SingletonView},
        vec::{VecBuffer},
        index::{buffer::IndexBuffer},
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView, make_label
        },
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        tree::NestedNode,
        Nested,
        Commander
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::{Vector2, Point2}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct DigitEditor {
    ctx: Arc<RwLock<Context>>,
    radix: u32,
    data: SingletonBuffer<Option<char>>,
    msg: VecBuffer<Message>,
}

impl Commander for DigitEditor {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                self.data.set(Some(*c));

                self.msg.clear();
                if c.to_digit(self.radix).is_none() {
                    let mut mb = IndexBuffer::new();
                    mb.insert_iter(vec![
                        (Point2::new(1, 0), make_label("invalid digit '")),
                        (Point2::new(2, 0), make_label(&format!("{}", *c))
                         .map_item(|_p,a| a.add_style_back(TerminalStyle::fg_color((140,140,250))))),
                        (Point2::new(3, 0), make_label("'"))
                    ]);
                    self.msg.push(crate::diagnostics::make_error(mb.get_port().flatten()));
                }
            }
            TerminalEvent::Input(Event::Key(Key::Backspace))
                | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                    self.data.set(None);
                    self.msg.clear();
                    self.msg.push(crate::diagnostics::make_warn(make_label("empty digit")));
                }
            _ => {}
        }
    }
}

impl DigitEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32) -> Self {
        DigitEditor {
            ctx,
            radix,
            data: SingletonBuffer::new(None),
            msg: VecBuffer::new(),
        }
    }

    pub fn into_node(self) -> NestedNode {
        let editor = Arc::new(RwLock::new(self));
        let ed = editor.read().unwrap();
        let r = ed.radix;

        NestedNode::new()
            .set_ctx(ed.ctx.clone())
            .set_cmd(editor.clone())
            .set_view(
                ed.data
                    .get_port()
                    .map(move |c| {
                        TerminalAtom::new(
                            c.unwrap_or('?'),
                            if c.unwrap_or('?').to_digit(r).is_some() {
                                TerminalStyle::fg_color((100, 140, 100))
                            } else {
                                //TerminalStyle::bg_color((90, 10, 10))
                                TerminalStyle::fg_color((200, 40, 40))
                            },
                        )
                    })
                    .to_grid()
            )
            .set_diag(
                ed.msg.get_port().to_sequence()
            )
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<u32>>> {
        let radix = self.radix;
        self.data.get_port().map(move |c| c?.to_digit(radix))
    }
}

pub struct PosIntEditor {
    radix: u32,
    digits: NestedNode
}

impl PosIntEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32) -> Self {
        PosIntEditor {
            radix,
            digits: PTYListEditor::new(
                ctx.clone(),
                TypeTerm::Type {
                    id: ctx.read().unwrap().get_typeid("Digit").unwrap(),
                    args: vec![
                        TypeTerm::Num(radix as i64)
                    ]
                },
                match radix {
                    16 => SeqDecorStyle::Hex,
                    _ => SeqDecorStyle::Plain
                },
                None,
                0
            ).into_node()
        }
    }

    pub fn into_node(self) -> NestedNode {
        self.digits
    }
/*
    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = u32>> {
        let radix = self.radix;
        self.digits
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
*/
}

