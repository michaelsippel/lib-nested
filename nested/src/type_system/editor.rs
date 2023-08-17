use {
    r3vi::{
        buffer::singleton::*,
        view::{singleton::*, sequence::*, OuterViewPort},
        projection::flatten_grid::*,
        projection::flatten_singleton::*
    },
    crate::{
        type_system::{Context, TypeID, TypeTerm, ReprTree, MorphismTypePattern},
        terminal::{TerminalEvent, TerminalStyle},
        editors::{sum::*, list::{ListCursorMode, ListEditor, PTYListStyle, PTYListController}},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander,
        PtySegment
    },
    termion::event::{Key},
    std::{sync::{Arc, RwLock}, any::Any},
    cgmath::{Vector2, Point2}
};

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Any,
    App,
    Ladder,
    Char,
    Num,
    AnySymbol,
    FunSymbol,
    VarSymbol,
}

pub struct TypeTermEditor {
    ctx: Arc<RwLock<Context>>,
    data: Arc<RwLock<ReprTree>>,

    // forward the editor to the node that references TypeTermEditor
    // will be removed once the node includes a spill buffer using which joins can be implemented
    editor: SingletonBuffer<
                Option< Arc<dyn Any + Send + Sync> >
            >,

    state: State,
    cur_node: SingletonBuffer< NestedNode >
}

impl TypeTermEditor {
    pub fn init_ctx(ctx: &mut Context) {
        ctx.add_list_typename("TypeTerm".into());
        ctx.add_list_typename("TypeSymbol".into());
        ctx.add_list_typename("TypeLadder".into());
        ctx.add_list_typename("TypeSymbol::Function".into());
        ctx.add_list_typename("TypeSymbol::Variable".into());
        ctx.add_list_typename("TypeSymbol::Literal::Num".into());
        ctx.add_list_typename("TypeSymbol::Literal::Char".into());

        ctx.add_morphism(
            MorphismTypePattern {
                src_tyid: ctx.get_typeid("List"),
                dst_tyid: ctx.get_typeid("TypeSymbol").unwrap()
            },
            Arc::new(
                |mut node, _dst_type:_| {
                    PTYListController::for_node( &mut node, Some(' '), None );
                    PTYListStyle::for_node( &mut node, ("","","") );

                    if let Some(v) = node.view {
                        node.view = Some(
                            v.map_item(|i,p| p.add_style_front(TerminalStyle::fg_color((220, 220, 200)))));
                    }

                    Some(node)
                }
            )
        );
        
        ctx.add_morphism(
            MorphismTypePattern {
                src_tyid: ctx.get_typeid("List"),
                dst_tyid: ctx.get_typeid("TypeSymbol::Function").unwrap()
            },
            Arc::new(
                |mut node, _dst_type:_| {
                    PTYListController::for_node( &mut node, None, None );
                    PTYListStyle::for_node( &mut node, ("","","") );

                    if let Some(v) = node.view {
                        node.view = Some(
                            v.map_item(|i,p| p.add_style_front(TerminalStyle::fg_color((220, 220, 220)))));
                    }

                    Some(node)
                }
            )
        );

        ctx.add_morphism(
            MorphismTypePattern {
                src_tyid: ctx.get_typeid("List"),
                dst_tyid: ctx.get_typeid("TypeSymbol::Variable").unwrap()
            },
            Arc::new(
                |mut node, _dst_type:_| {
                    PTYListController::for_node( &mut node, None, None );
                    PTYListStyle::for_node( &mut node, ("","","") );

                    if let Some(v) = node.view {
                        node.view = Some(
                            v.map_item(|i,p| p.add_style_front(TerminalStyle::fg_color((5, 120, 240)))));
                    }

                    Some(node)
                }
            )
        );

        ctx.add_node_ctor(
            "TypeTerm", Arc::new(
                |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                    Some(TypeTermEditor::new_node(ctx, depth))
                }
            )
        );

        ctx.add_morphism(
            MorphismTypePattern {
                src_tyid: ctx.get_typeid("List"),
                dst_tyid: ctx.get_typeid("TypeTerm").unwrap()
            },
            Arc::new(
                move |mut node, _dst_type:_| {
                    eprintln!("morphism to typeterm");
                    PTYListController::for_node( &mut node, Some(' '), None );
                    PTYListStyle::for_node( &mut node, ("","","") );
                    let mut new_node = TypeTermEditor::with_node( node.ctx.clone(), node.depth.get(), node.clone(), State::Any );
                    Some(new_node)
                }
            )
        );
    }

    pub fn from_type_term(ctx: Arc<RwLock<Context>>, depth: usize, term: &TypeTerm) -> NestedNode {
        let mut node = TypeTermEditor::new_node(ctx.clone(), depth);
        node.goto(TreeCursor::home());

        match term {
            TypeTerm::TypeID( tyid ) => {
                let mut editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state(match tyid {
                    TypeID::Fun(_) => State::FunSymbol,
                    TypeID::Var(_) => State::VarSymbol
                });

                let typename = ctx.read().unwrap().get_typename(&tyid).unwrap_or("UNKNOWN TYPE".into());
                for x in typename.chars()
                {
                    node.send_cmd_obj(
                        ReprTree::from_char( &ctx, x )
                    );
                }
            },

            TypeTerm::App( args ) => {
                let mut editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state( State::App );

                for x in args.iter() {
                    let mut arg_node = TypeTermEditor::from_type_term( ctx.clone(), depth+1, x );

                    eprintln!("add node arg!");
                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            (&ctx, "( NestedNode )"),
                            SingletonBuffer::new(arg_node).get_port().into()
                        )
                    );
                }
            }

            TypeTerm::Ladder( args ) => {
                let mut editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state( State::Ladder );

                for x in args.iter() {
                    let mut arg_node = TypeTermEditor::from_type_term( ctx.clone(), depth+1, x );

                    eprintln!("add node arg!");
                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            (&ctx, "( NestedNode )"),
                            SingletonBuffer::new(arg_node).get_port().into()
                        )
                    );
                }
            }

            TypeTerm::Num( n ) => {
                let mut editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");

                let mut int_edit = crate::editors::integer::PosIntEditor::from_u64(node.ctx.clone(), 10, *n as u64);
                let mut node = int_edit.into_node();

                editor.write().unwrap().editor.set(node.editor.get());
                editor.write().unwrap().cur_node.set(node);
                editor.write().unwrap().state = State::Num;                
            }

            TypeTerm::Char( c ) => {
                let mut editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");

                editor.write().unwrap().set_state( State::Char );
                editor.write().unwrap().send_cmd_obj(ReprTree::from_char(&ctx, *c));
            }
            
            _ => {}
        }

        node.goto(TreeCursor::none());
        node
    }
    
    fn set_state(&mut self, new_state: State) {
        let old_node = self.cur_node.get();
        let mut node = match new_state {
            State::Char => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( Char )").into(), 0 ).unwrap();
                let mut grid = r3vi::buffer::index_hashmap::IndexBuffer::new();

                grid.insert_iter(
                    vec![
                        (Point2::new(0,0), crate::terminal::make_label("'")),
                        (Point2::new(1,0), node.view.clone().unwrap()),
                        (Point2::new(2,0), crate::terminal::make_label("'")),
                    ]
                );
                
                node.close_char.set(Some('\''));
                node.view = Some(
                    grid.get_port()
                        .flatten()
                );

                self.data.write().unwrap().insert_leaf(
                    vec![].into_iter(),
                    node.data.read().unwrap()
                        .get_port::<dyn SingletonView<Item = char>>().unwrap()
                        .map(
                            |c| TypeTerm::Char(c)
                        )
                        .into()
                );
                
                node
            }
            State::App => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List TypeTerm )").into(), 0 ).unwrap();

                PTYListController::for_node( &mut node, Some(' '), Some('>') );
                PTYListStyle::for_node( &mut node, ("<"," ",">") );

                self.data.write().unwrap().insert_leaf(
                    vec![].into_iter(),
                    node.data.read().unwrap()
                        .get_port::<dyn SequenceView<Item = NestedNode>>().unwrap()
                        .map(
                            |node| {
                                node.data.read().unwrap().get_port::<dyn SingletonView<Item = TypeTerm>>().unwrap()
                            }
                        )
                        .into()
                );
                
                node
            }
            State::Ladder => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List TypeTerm )").into(), 0 ).unwrap();

                PTYListController::for_node( &mut node, Some('~'), None );
                PTYListStyle::for_node( &mut node, ("","~","") );

                self.data.write().unwrap().insert_leaf(
                    vec![].into_iter(),
                    node.data.read().unwrap()
                        .get_port::<dyn SequenceView<Item = NestedNode>>().unwrap()
                        .map(
                            |node| {
                                node.data.read().unwrap().get_port::<dyn SingletonView<Item = TypeTerm>>().unwrap()
                            }
                        )
                        .into()
                );
                
                node
            }
            State::AnySymbol => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap();
                node = node.morph( (&self.ctx, "( TypeSymbol )").into() );
                node
            },
            State::FunSymbol => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap();
                node = node.morph( (&self.ctx, "( TypeSymbol::Function )").into() );
                node
            },
            State::VarSymbol => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap();
                node = node.morph( (&self.ctx, "( TypeSymbol::Variable )").into() );
                node
            }
            State::Num => {
                Context::make_node( &self.ctx, (&self.ctx, "( PosInt 10 BigEndian )").into(), 0 ).unwrap()
            }
            _ => {
                old_node
            }
        };

        node.goto(TreeCursor::home());

        self.editor.set(node.editor.get());
        self.cur_node.set(node);
        self.state = new_state;
    }

    pub fn new_node(ctx: Arc<RwLock<Context>>, depth: usize) -> NestedNode {
        let mut symb_node = Context::make_node( &ctx, (&ctx, "( List Char )").into(), 0 ).unwrap();
        symb_node = symb_node.morph( (&ctx, "( TypeSymbol::Variable )").into() );

        Self::with_node(
            ctx.clone(),
            depth,
            symb_node,
            State::Any
        )
    }

    fn with_node(ctx: Arc<RwLock<Context>>, depth: usize, node: NestedNode, state: State) -> NestedNode {
        let buffer = SingletonBuffer::<Option<TypeTerm>>::new( None );

        let data = Arc::new(RwLock::new(ReprTree::new(
            (&ctx, "( TypeTerm )")
        )));

        let mut editor = TypeTermEditor {
            ctx: ctx.clone(),
            state,
            data: data.clone(),
            cur_node: SingletonBuffer::new(node),
            editor: SingletonBuffer::new(None)
        };

        let view = editor.cur_node
            .get_port()
            .map(
                |node| {
                    match node.view.clone() {
                        Some(v) => {
                            v
                        }
                        None => {
                            r3vi::view::ViewPort::new().into_outer()
                        }
                    }
                }
            )
            .to_grid()
            .flatten();

        let cc = editor.cur_node.get().close_char;
        let editor = Arc::new(RwLock::new(editor));

        let mut node = NestedNode::new(ctx, data, depth)
            .set_view(view)
            .set_nav(editor.clone())
            .set_cmd(editor.clone())
            .set_editor(editor.clone());

        node.close_char.set(cc.get());
        editor.write().unwrap().editor = node.editor.clone();
        
        node
    }

    fn get_typeterm(&self) -> Option<TypeTerm> {
        match self.state {
            State::Any => None,

            State::AnySymbol => {
                /*
                let x = self.data.descend_ladder(vec![
                    (&ctx, "( FunctionID )").into(),
                    (&ctx, "( Symbol )").into(),
                    (&ctx, "( List Char )").into(),
                ].into_iter());

                let fun_name = /* x...*/ "PosInt";
                let fun_id = self.ctx.read().unwrap().get_typeid( fun_name );

                self.data.add_repr(
                    vec![
                        (&ctx, "( FunctionID )").into(),
                        (&ctx, "( MachineInt )").into()
                    ]
                );
*/
                Some(TypeTerm::new(TypeID::Fun(0)))
            },
            State::App => {                
                Some(TypeTerm::new(TypeID::Fun(0)))
            },

            State::Char => {
                Some(TypeTerm::Char('c'))
            }
            State::Num => {
                Some(TypeTerm::Num(44))
            }
            _ => {None}
        }
    }
}

impl TreeNav for TypeTermEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.cur_node.get().get_cursor()
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        // fixme this is wrong
        self.cur_node.get().get_addr_view()
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        // this is wrong
        self.cur_node.get().get_mode_view()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        self.cur_node.get().get_cursor_warp()
    }

    fn get_max_depth(&self) -> usize {
        self.cur_node.get().get_max_depth()
    }

    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        self.cur_node.get_mut().goby(dir)
    }

    fn goto(&mut self, new_cur: TreeCursor) -> TreeNavResult {
        self.cur_node.get_mut().goto(new_cur)
    }
}

impl ObjCommander for TypeTermEditor {
    fn send_cmd_obj(&mut self, co: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_obj = co.clone();
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        if cmd_type == (&self.ctx, "( Char )").into() {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let c = cmd_view.get();

                match &self.state {
                    State::Any => {
                        match c {
                            '<' => {
                                self.set_state( State::App );
                                TreeNavResult::Continue
                            }
                            '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                                self.set_state( State::Num );
                                self.cur_node.get_mut().send_cmd_obj( co );
                                TreeNavResult::Continue
                            }
                            '\'' => {
                                self.set_state( State::Char );
                                TreeNavResult::Continue
                            }
                            _ => {
                                self.set_state( State::AnySymbol );
                                self.cur_node.get_mut().goto(TreeCursor::home());
                                self.cur_node.get_mut().send_cmd_obj( co )
                            }
                        }
                    }

                    State::Char => {
                        match c {
                            '\'' => {
                                self.cur_node.get_mut().goto(TreeCursor::none());
                                TreeNavResult::Exit
                            }
                            _ => {
                                self.cur_node.get_mut().send_cmd_obj( co )
                            }
                        }
                    }

                    _ => {
                        self.cur_node.get_mut().send_cmd_obj( co )
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            match &self.state {
                State::Any => {
                    eprintln!("undefined comd object set to listl");
                    self.set_state( State::App );
                    self.cur_node.get_mut().goto(TreeCursor::home());
                }
                _ => {}
            }

            match self.state {
                State::App => {
                    self.cur_node.get().send_cmd_obj( co )
                },
                State::Ladder => {
                    self.cur_node.get().send_cmd_obj( co )
                },
                _ => {
                    eprintln!("undefined cmd object");
                    TreeNavResult::Exit
                }
            }
        }
    }
}

