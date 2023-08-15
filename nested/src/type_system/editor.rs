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
    Char,
    Num,
    List,
    Symbol,
    Fun,
    Var,
}

pub struct TypeTermEditor {
    ctx: Arc<RwLock<Context>>,

    data: Arc<RwLock<ReprTree>>,

    // forward the editor to the node that references TypeTermEditor
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
        ctx.add_list_typename("TypeSymbol::Function".into());
        ctx.add_list_typename("TypeSymbol::Variable".into());

        let pattern = MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("TypeSymbol::Function").unwrap()
        };

        ctx.add_morphism(pattern,
                         Arc::new(
                             |mut node, _dst_type:_| {
                                 PTYListController::for_node( &mut node, None, None );
                                 PTYListStyle::for_node( &mut node, ("","","") );

                                 if let Some(v) = node.view {
                                     node.view = Some(
                                         v.map_item(|i,p| p.add_style_front(TerminalStyle::fg_color((220, 220, 0)))));
                                 }

                                 Some(node)
                             }
                         )
        );

        let pattern = MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("TypeSymbol::Variable").unwrap()
        };

        ctx.add_morphism(pattern,
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

/*
        ctx.add_list_typename("TypeLadder".into());
        ctx.add_node_ctor(
            "TypeLadder", Arc::new(
                |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {

                }
            )
        );
*/
        let pattern = MorphismTypePattern {
            src_tyid: ctx.get_typeid("List"),
            dst_tyid: ctx.get_typeid("TypeTerm").unwrap()
        };

        ctx.add_morphism(pattern,
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
/*
    fn from_type_term(term: TypeTerm) -> TypeTermEditor {
        match term {
            TypeTerm::
        }
    }
*/
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
            State::List => {
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
            State::Symbol => {
                Context::make_node( &self.ctx, (&self.ctx, "( Symbol )").into(), 0 ).unwrap()
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
        Self::with_node(ctx.clone(), depth, Context::make_node( &ctx, (&ctx, "( Symbol )").into(), 0 ).unwrap(), State::Any)
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

            State::Symbol => {
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
            State::List => {                
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
                                self.set_state( State::List );
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
                                self.set_state( State::Symbol );
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

                    State::List => {
                        self.cur_node.get_mut().send_cmd_obj( co )
                        /*
                        match self.cur_node.get_mut().send_cmd_obj( co ) {
                            TreeNavResult::Continue => {
                                TreeNavResult::Continue
                            }

                            TreeNavResult::Exit => {
                                match c {
                                    '>' => {
                                        let cur = self.get_cursor();

                                        if cur.tree_addr.len() > 2 {
                                            self.goto(
                                                TreeCursor {
                                                    leaf_mode: ListCursorMode::Insert,
                                                    tree_addr: vec![ cur.tree_addr.get(0).unwrap_or(&0)+1 ]
                                                }
                                            );
                                            TreeNavResult::Continue
                                        } else {
                                            TreeNavResult::Exit
                                        }
                                    }
                                    _ => {
                                        TreeNavResult::Exit
                                    }
                            }
                            }
                    }
                            */
                    }

                    _ => {
                        self.cur_node.get_mut().send_cmd_obj( co )
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
//            eprintln!("undefined comd object");
            match &self.state {
                State::Any => {
                    eprintln!("undefined comd object set to listl");                    
                    self.set_state( State::List );
                    self.cur_node.get_mut().goto(TreeCursor::home());
                }
                _ => {}
            }

            self.cur_node.get().cmd.get().unwrap().write().unwrap().send_cmd_obj( co )
        }
    }
}

