mod ctx;

pub use ctx::init_ctx;

use {
    r3vi::{
        buffer::singleton::*,
        view::{singleton::*, sequence::*, OuterViewPort}
    },
    crate::{
        type_system::{Context, TypeID, TypeTerm, ReprTree},
        editors::{list::{ListCursorMode, ListEditor, ListCmd}},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander
    },
    std::{sync::{Arc, RwLock}, any::Any},
    cgmath::{Vector2}
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    Any,
    Num,
    Char,
    AnySymbol,
    FunSymbol,
    VarSymbol,
    App,
    Ladder,
}

pub struct TypeTermEditor {
    ctx: Arc<RwLock<Context>>,
    data: Arc<RwLock<ReprTree>>,

    // forward the editor to the node that references TypeTermEditor
    // will be removed once the node includes a spill buffer using which joins can be implemented
    editor: SingletonBuffer<
                Option< Arc<dyn Any + Send + Sync> >
            >,

    close_char: SingletonBuffer<Option<char>>,

    state: State,
    cur_node: SingletonBuffer< NestedNode >
}

impl TypeTermEditor {
    pub fn from_type_term(ctx: Arc<RwLock<Context>>, depth: usize, term: &TypeTerm) -> NestedNode {
        let mut node = TypeTermEditor::new_node(ctx.clone(), depth);
        node.goto(TreeCursor::home());

        match term {
            TypeTerm::TypeID( tyid ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state(match tyid {
                    TypeID::Fun(_) => State::FunSymbol,
                    TypeID::Var(_) => State::VarSymbol
                });

                let typename = ctx.read().unwrap().get_typename(&tyid).unwrap_or("UNNAMED TYPE".into());
                for x in typename.chars()
                {
                    node.send_cmd_obj(
                        ReprTree::from_char( &ctx, x )
                    );
                }
            },

            TypeTerm::App( args ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state( State::App );

                let parent_ctx = editor.read().unwrap().cur_node.get().ctx.clone();

                for x in args.iter() {                    
                    let arg_node = TypeTermEditor::from_type_term( parent_ctx.clone(), depth+1, x );

                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            (&ctx, "( NestedNode )"),
                            SingletonBuffer::new(arg_node).get_port().into()
                        )
                    );
                }
            }

            TypeTerm::Ladder( args ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                editor.write().unwrap().set_state( State::Ladder );

                let parent_ctx = editor.read().unwrap().cur_node.get().ctx.clone();

                for x in args.iter() {
                    let arg_node = TypeTermEditor::from_type_term( parent_ctx.clone(), depth+1, x );

                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            (&ctx, "( NestedNode )"),
                            SingletonBuffer::new(arg_node).get_port().into()
                        )
                    );
                }
            }

            TypeTerm::Num( n ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");

                let parent_ctx = editor.read().unwrap().cur_node.get().ctx.clone();

                let int_edit = crate::editors::integer::PosIntEditor::from_u64(parent_ctx, 10, *n as u64);
                let node = int_edit.into_node();

                editor.write().unwrap().editor.set(node.editor.get());
                editor.write().unwrap().cur_node.set(node);
                editor.write().unwrap().state = State::Num;
            }

            TypeTerm::Char( c ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");

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
            State::App => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Type )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::App )").into() )
            }
            State::Ladder => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Type )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Ladder )").into() )
            }
            State::AnySymbol => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Sym )").into() )
            },
            State::FunSymbol => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Sym::Fun )").into() )
            },
            State::VarSymbol => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Sym::Var )").into() )
            }
            State::Num => {
                crate::editors::integer::PosIntEditor::new(self.ctx.clone(), 10)
                    .into_node()
                    .morph( (&self.ctx, "( Type::Lit::Num )").into() )
            }
            State::Char => {
                Context::make_node( &self.ctx, (&self.ctx, "( Char )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Lit::Char )").into() )
            }
            _ => {
                old_node
            }
        };

        node.goto(TreeCursor::home());

        let editor = node.editor.get();

        self.editor.set(editor);
        self.close_char.set(node.close_char.get());

        self.cur_node.set(node);
        self.state = new_state;
    }

    pub fn new_node(ctx: Arc<RwLock<Context>>, depth: usize) -> NestedNode {        
        let mut symb_node = Context::make_node( &ctx, (&ctx, "( List Char )").into(), 0 ).unwrap();
        symb_node = symb_node.morph( (&ctx, "( Type::Sym )").into() );

        Self::with_node(
            ctx.clone(),
            depth,
            symb_node,
            State::Any
        )
    }

    fn with_node(ctx: Arc<RwLock<Context>>, depth: usize, node: NestedNode, state: State) -> NestedNode {
        let _buffer = SingletonBuffer::<Option<TypeTerm>>::new( None );

        let data = Arc::new(RwLock::new(ReprTree::new(
            (&ctx, "( Type )")
        )));

        let editor = TypeTermEditor {
            ctx: ctx.clone(),
            state,
            data: data.clone(),
            cur_node: SingletonBuffer::new(node),
            editor: SingletonBuffer::new(None),
            close_char: SingletonBuffer::new(None)
        };

        let view = editor.cur_node
            .get_port()
            .map(|node| {
                node.view.clone().unwrap_or(r3vi::view::ViewPort::new().into_outer())
            })
            .to_grid()
            .flatten();
        let cc = editor.cur_node.get().close_char;
        let editor = Arc::new(RwLock::new(editor));

        let mut node = NestedNode::new(ctx, data, depth)
            .set_view(view)
            .set_nav(editor.clone())
            .set_cmd(editor.clone())
            .set_editor(editor.clone());

        editor.write().unwrap().close_char = node.close_char.clone();
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

        if cmd_obj.get_type().clone() == (&self.ctx, "( Char )").into() {
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
                            '~' => {
                                TreeNavResult::Exit
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
                        match self.cur_node.get_mut().send_cmd_obj( co ) {
                            TreeNavResult::Exit => {
                                match c {
                                    '~' => {
                                        // todo:
                                        
                                        // in case previous item is not ladder
                                        // create new intermediary ladder-node
                                        // with the previous item as one step

                                        // in case previous item is ladder
                                        // goto end

                                        eprintln!("TypeEdit: child exit ~");
                                    }
                                    _ => {}
                                }

                                TreeNavResult::Exit
                            }
                            TreeNavResult::Continue => {
                                TreeNavResult::Continue
                            }
                        }
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            match &self.state {
                State::Any => {
                    eprintln!("undefined comd object set to ladder");
                    self.set_state( State::Ladder );
                    self.cur_node.get_mut().goto(TreeCursor::home());
                    let res = self.cur_node.get().cmd.get().unwrap().write().unwrap().send_cmd_obj( co );
                    self.cur_node.get_mut().goto(TreeCursor::none());
                    res
                }
                _ => {
                    self.cur_node.get().cmd.get().unwrap().write().unwrap().send_cmd_obj( co )
                }
            }
        }
    }
}

