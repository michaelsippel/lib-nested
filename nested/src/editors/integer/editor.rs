use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
        },
        buffer::{
            singleton::*,
            vec::*,
            index_hashmap::*
        }
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::{PTYListEditor},
        terminal::{
            TerminalAtom, TerminalEvent, TerminalStyle, make_label
        },
        diagnostics::{Message},
        tree::NestedNode,
        commander::Commander
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::{Point2}
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

    pub fn into_node(self, depth: usize) -> NestedNode {
        let data = self.get_data();        
        let editor = Arc::new(RwLock::new(self));
        let mut ed = editor.write().unwrap();
        let r = ed.radix;

        NestedNode::new(depth)
            .set_ctx(ed.ctx.clone())
            .set_cmd(editor.clone())
            .set_data(data)
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

    pub fn get_type(&self) -> TypeTerm {
        TypeTerm::Type {
            id: self.ctx.read().unwrap().get_typeid("Digit").unwrap(),
            args: vec![
                TypeTerm::Num(self.radix as i64)
            ]
        }
    }

    pub fn get_data(&self) -> Arc<RwLock<ReprTree>> {
        let data_view = self.get_data_port();
        ReprTree::ascend(
            &ReprTree::new_leaf(
                self.ctx.read().unwrap().type_term_from_str("( u32 )").unwrap(),
                data_view.into()
            ),
            self.get_type()
        )
    }
}

pub struct PosIntEditor {
    radix: u32,
    digits: NestedNode
}

impl PosIntEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32) -> Self {
        let mut editor = PTYListEditor::new(
            ctx.clone(),
            TypeTerm::Type {
                id: ctx.read().unwrap().get_typeid("Digit").unwrap(),
                args: vec![
                    TypeTerm::Num(radix as i64)
                ]
            },
            None,
            0
        );

            let view = editor.pty_view((
                match radix {
                    2 => "0d".into(),
                    16 => "0x".into(),
                    _ => "".into()
                },
                "".into(),
                "".into()));
            let mut node = editor.into_node().set_view(view);

        // Set Type
        let data = node.data.clone().unwrap();
        node = node.set_data(ReprTree::ascend(
            &data,
            TypeTerm::Type {
                id: ctx.read().unwrap().get_typeid("PosInt").unwrap(),
                args: vec![
                    TypeTerm::Num(radix as i64),
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("BigEndian").unwrap(),
                        args: vec![]
                    }
                ]
            }
        ));

        PosIntEditor {
            radix,
            digits: node
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

