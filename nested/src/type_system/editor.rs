use {
    r3vi::{
        buffer::singleton::*,
        view::{singleton::*, sequence::*, OuterViewPort},
        projection::flatten_grid::*,
        projection::flatten_singleton::*
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree, MorphismTypePattern},
        terminal::{TerminalEvent},
        editors::{sum::*, list::{ListCursorMode, ListEditor, PTYListStyle, PTYListController}},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander,
        PtySegment
    },
    termion::event::{Key},
    std::{sync::{Arc, RwLock}},
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

    state: State,
    cur_node: SingletonBuffer<NestedNode>
}

impl TypeTermEditor {
    pub fn init_ctx(ctx: &mut Context) {
        ctx.add_list_typename("TypeTerm".into());
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
                             |mut node, _dst_type:_| {
                                 //eprintln!("morphism to typeterm");

                                 PTYListController::for_node( &mut node, Some(' '), None );
                                 PTYListStyle::for_node( &mut node, ("","","") );

                                 let mut new_node = TypeTermEditor::with_node( node.ctx.clone().unwrap(), node.depth, node.clone(), State::Any );

                                 let item_nodes = node.get_edit::<ListEditor>().clone().unwrap();
                                 let item_nodes = item_nodes.read().unwrap();

                                 for i in 0..item_nodes.data.len() {
                                     if let Some(x) = item_nodes.data.get(i).data {
                                         //eprintln!("item with {:?}", x);
                                         //let c = x.read().unwrap().get_view::<dyn SingletonView<Item = NestedNode>>().unwrap().get();
                                         new_node.send_cmd_obj(
                                             ReprTree::from_char(&new_node.ctx.as_ref().unwrap(), 'x')
                                         );
                                         //new_node.send_cmd_obj(c);
                                     }
                                 }

                                 if item_nodes.data.len() > 0 {
                                     new_node.goto(TreeCursor::home());
                                 }

                                 Some(new_node)
                             }
                         )
        );
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
                
                node.close_char = Some('\'');
                node.view = Some(
                    grid.get_port()
                        .flatten()
                );
                node
            }
            State::List => {
                let mut node = Context::make_node( &self.ctx, (&self.ctx, "( List TypeTerm )").into(), 0 ).unwrap();

                PTYListController::for_node( &mut node, Some(' '), Some('>') );
                PTYListStyle::for_node( &mut node, ("<"," ",">") );

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

        self.cur_node.set(node);
        self.state = new_state;
    }

    pub fn new_node(ctx: Arc<RwLock<Context>>, depth: usize) -> NestedNode {
        Self::with_node(ctx.clone(), depth, Context::make_node( &ctx, (&ctx, "( Symbol )").into(), 0 ).unwrap(), State::Any)
    }

    fn with_node(ctx: Arc<RwLock<Context>>, depth: usize, node: NestedNode, state: State) -> NestedNode {
        let mut editor = TypeTermEditor {
            ctx: ctx.clone(),
            state,
            cur_node: SingletonBuffer::new(node)
        };

        let ed_view = editor.cur_node
            .get_port()
            .map(
                |node|
                match node.editor {
                    Some(e) => {
                        e
                    },
                    None => {
                        r3vi::buffer::singleton::SingletonBuffer::new(None).get_port()
                    }
                }
            )
            .flatten();

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

        let mut node = NestedNode::new(depth)
            .set_ctx(ctx)
            .set_view( view )
            .set_nav(editor.clone())
            .set_cmd(editor.clone());

        node.editor = Some(ed_view);
        //node.editor.unwrap().get_view().unwrap().get().unwrap()

        node
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

        let char_type = (&self.ctx, "( Char )").into();

        if cmd_type == char_type {
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
                    }

                    _ => {
                        self.cur_node.get_mut().send_cmd_obj( co )
                    }
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            self.cur_node.get_mut().send_cmd_obj( co )
        }
    }
}
