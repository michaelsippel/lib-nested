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
        editors::list::{ListEditor, PTYListController, PTYListStyle},
        terminal::{
            TerminalAtom, TerminalEvent, TerminalStyle, make_label
        },
        diagnostics::{Message},
        tree::{NestedNode, TreeNavResult},
        commander::ObjCommander
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

impl ObjCommander for DigitEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        let char_type = (&self.ctx, "( Char )").into();
        //let _term_event_type = (&ctx, "( TerminalEvent )").into();

        if cmd_type == char_type {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let c = cmd_view.get();
                self.data.set(Some(c));

                self.msg.clear();

                if self.ctx.read().unwrap().meta_chars.contains(&c) {
                    return TreeNavResult::Exit;
                }
                else if c.to_digit(self.radix).is_none() {
                    let mut mb = IndexBuffer::new();
                    mb.insert_iter(vec![
                        (Point2::new(1, 0), make_label("invalid digit '")),
                        (Point2::new(2, 0), make_label(&format!("{}", c))
                         .map_item(|_p,a| a.add_style_back(TerminalStyle::fg_color((140,140,250))))),
                        (Point2::new(3, 0), make_label("'"))
                    ]);
                    self.msg.push(crate::diagnostics::make_error(mb.get_port().flatten()));
                }
            }
        }

        TreeNavResult::Continue
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
        let ed = editor.write().unwrap();
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
            id: self.ctx.read().unwrap().get_fun_typeid("Digit").unwrap(),
            args: vec![
                TypeTerm::Num(self.radix as i64).into()
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
        let mut node = Context::make_node(&ctx, (&ctx, format!("( List ( Digit {} ) )", radix).as_str()).into(), 0).unwrap();

        PTYListController::for_node( &mut node, Some(' '), None );
        PTYListStyle::for_node( &mut node,
            (
                match radix {
                    2 => "0d".into(),
                    16 => "0x".into(),
                    _ => "".into()
                },
                "".into(),
                "".into()
            )
        );

        // Set Type
        let data = node.data.clone().unwrap();
        node = node.set_data(ReprTree::ascend(
            &data,
            TypeTerm::Type {
                id: ctx.read().unwrap().get_fun_typeid("PosInt").unwrap(),
                args: vec![
                    TypeTerm::Num(radix as i64).into(),
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_fun_typeid("BigEndian").unwrap(),
                        args: vec![]
                    }.into()
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

