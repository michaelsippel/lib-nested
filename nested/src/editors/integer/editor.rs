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
        editors::list::{ListCmd, PTYListController, PTYListStyle},
        terminal::{
            TerminalAtom, TerminalStyle, make_label
        },
        diagnostics::{Message},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander
    },
    std::sync::Arc,
    std::sync::RwLock,
    std::iter::FromIterator,
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

        if cmd_type == (&self.ctx, "( Char )").into() {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let c = cmd_view.get();

                self.msg.clear();

                if self.ctx.read().unwrap().meta_chars.contains(&c) {
                    eprintln!("digitedit: meta char");
                    return TreeNavResult::Exit;

                } else if c.to_digit(self.radix).is_none() {
                    /* in case the character c is not in the range of digit-chars,
                       add a message to the diagnostics view
                     */

                    let message = IndexBuffer::from_iter(vec![
                        (Point2::new(1, 0), make_label("invalid digit '")),
                        (Point2::new(2, 0), make_label(&format!("{}", c))
                         .map_item(|_p,a| a.add_style_back(TerminalStyle::fg_color((140,140,250))))),
                        (Point2::new(3, 0), make_label("'"))
                    ]);

                    self.msg.push(crate::diagnostics::make_error(message.get_port().flatten()));
                    self.data.set(Some(c));
                } else {
                    self.data.set(Some(c));
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

        NestedNode::new(ed.ctx.clone(), data, depth)
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

    pub fn get_type(&self) -> TypeTerm {
        TypeTerm::TypeID(self.ctx.read().unwrap().get_typeid("Digit").unwrap())
    }

    pub fn get_data(&self) -> Arc<RwLock<ReprTree>> {
        ReprTree::ascend(
            &ReprTree::new_leaf(
                self.ctx.read().unwrap().type_term_from_str("( Seq u32 )").unwrap(),
                self.get_data_port().into()
            ),
            self.get_type()
        )
    }
}


pub struct PosIntEditor {
    radix: u32,
    digits: NestedNode,

    // todo: endianness
}

impl PosIntEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32) -> Self {
        let mut node = Context::make_node(
            &ctx,
            (&ctx, format!("( List ( Digit {} ) )", radix).as_str()).into(),
            0
        ).unwrap();

        // Set Type
        node.data = ReprTree::ascend(
            &node.data.clone(),
            TypeTerm::App(vec![
                TypeTerm::TypeID(ctx.read().unwrap().get_typeid("PosInt").unwrap()),
                TypeTerm::Num(radix as i64).into(),
                TypeTerm::TypeID(ctx.read().unwrap().get_typeid("BigEndian").unwrap())
            ]
        ));

        PTYListController::for_node( &mut node, Some(' '), None );
        PTYListStyle::for_node( &mut node,
            (
                match radix {
                    2 => "0b".into(),
                    8 => "0o".into(),
                    10 => "0d".into(),
                    16 => "0x".into(),
                    _ => "".into()
                },
                "".into(),
                "".into()
            )
        );

        PosIntEditor {
            radix,
            digits: node
        }
    }

    pub fn from_u64(ctx: Arc<RwLock<Context>>, radix: u32, value: u64) -> Self {
        let mut edit = PosIntEditor::new(ctx, radix);
        edit.set_value_u64( value );
        edit
    }

    pub fn set_value_u64(&mut self, mut value: u64) {
        self.digits.send_cmd_obj(ListCmd::Clear.into_repr_tree(&self.digits.ctx));

        while value > 0 {
            let digit_val = (value % self.radix as u64) as u32;
            value /= self.radix as u64;

            // if BigEndian
            self.digits.goto(TreeCursor::home());

            self.digits.send_cmd_obj(ReprTree::from_char(&self.digits.ctx, char::from_digit(digit_val, self.radix).expect("invalid digit"))); 
        }
        self.digits.goto(TreeCursor::none());
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

