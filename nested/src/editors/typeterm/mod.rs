mod ctx;
mod nav;
mod cmd;

pub use ctx::init_ctx;

use {
    r3vi::{
        buffer::{singleton::*, vec::*},
        view::{singleton::*, sequence::*, OuterViewPort}
    },
    crate::{
        type_system::{Context, TypeID, TypeTerm, ReprTree},
        editors::{list::{ListCursorMode, ListEditor, ListCmd}},
        tree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        commander::ObjCommander
    },
    std::{sync::{Arc, RwLock, Mutex}, any::Any},
    cgmath::{Vector2}
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(super) enum State {
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

    // shared with NestedNode that contains self
    ctx: Arc<RwLock<Context>>,
    data: Arc<RwLock<ReprTree>>,
    close_char: SingletonBuffer<Option<char>>,
    spillbuf: Arc<RwLock<Vec<Arc<RwLock<NestedNode>>>>>,

    // editing/parsing state
    state: State,

    // child node
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
                for x in typename.chars() {
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

                let mut editor = editor.write().unwrap();
                editor.cur_node.set(
                    crate::editors::integer::PosIntEditor::from_u64(parent_ctx, 10, *n as u64)
                        .into_node()
                );
                editor.state = State::Num;
            }

            TypeTerm::Char( c ) => {
                let editor = node.get_edit::<TypeTermEditor>().expect("typ term edit");
                let mut editor = editor.write().unwrap();
                editor.set_state( State::Char );
                editor.send_cmd_obj(ReprTree::from_char(&ctx, *c));
            }
            
            _ => {}
        }

        node.goto(TreeCursor::none());
        node
    }
    
    fn set_state(&mut self, new_state: State) {        
        let old_node = self.cur_node.get();

        let mut node = match new_state {
            State::Any => {
                Context::make_node( &self.ctx, (&self.ctx, "( List Char )").into(), 0 ).unwrap()
                    .morph( (&self.ctx, "( Type::Sym )").into() )

            }
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
        self.close_char.set(node.close_char.get());
        self.cur_node.set(node);
        self.state = new_state;
    }

    pub fn new_node(ctx: Arc<RwLock<Context>>, depth: usize) -> NestedNode {
        let ctx : Arc<RwLock<Context>> = Arc::new(RwLock::new(Context::with_parent(Some(ctx))));
        ctx.write().unwrap().meta_chars.push('~');

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
            data: data.clone(),
            state,
            cur_node: SingletonBuffer::new(node),
            close_char: SingletonBuffer::new(None),
            spillbuf: Arc::new(RwLock::new(Vec::new()))
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
        node.spillbuf = editor.read().unwrap().spillbuf.clone();
        
        node
    }

    fn forward_spill(&mut self) {
        let node = self.cur_node.get();
        let mut buf = node.spillbuf.write().unwrap();
        for n in buf.iter() {
            self.spillbuf.write().unwrap().push(n.clone());
        }
        buf.clear();
    }

    fn send_child_cmd(&mut self, cmd: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let res = self.cur_node.get_mut().send_cmd_obj( cmd );
        self.forward_spill();
        res
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

    pub fn normalize_empty(&mut self) {
        eprintln!("normalize singleton");
        let mut subladder_list_node = self.cur_node.get().clone();
        let mut subladder_list_edit = subladder_list_node.get_edit::<ListEditor>().unwrap();

        let subladder_list_edit = subladder_list_edit.read().unwrap();
        if subladder_list_edit.data.len() == 0 {

            self.set_state( State::Any );
        }
    }

    
    /* unwrap a ladder if it only contains one element
     */
    pub fn normalize_singleton(&mut self) {
        eprintln!("normalize singleton");
        let mut subladder_list_node = self.cur_node.get().clone();
        let mut subladder_list_edit = subladder_list_node.get_edit::<ListEditor>().unwrap();

        let subladder_list_edit = subladder_list_edit.read().unwrap();
        if subladder_list_edit.data.len() == 1 {
            let it_node = subladder_list_edit.data.get(0);
            let it_node = it_node.read().unwrap();
            if it_node.get_type() == (&self.ctx, "( Type )").into() {
                let other_tt = it_node.get_edit::<TypeTermEditor>().unwrap();

                let mut other_tt = other_tt.write().unwrap();

                other_tt.normalize_singleton();

                self.close_char.set(other_tt.close_char.get());
                self.cur_node.set(other_tt.cur_node.get());
                self.state = other_tt.state;
            }
        } else {
        }
    }

    /* flatten ladder of ladders into one ladder editor
     */
    pub fn normalize_nested_ladder(&mut self) {
        let mut subladder_list_node = self.cur_node.get().clone(); 
        let mut subladder_list_edit = subladder_list_node.get_edit::<ListEditor>().unwrap();

        let item = subladder_list_edit.write().unwrap().get_item().clone();

        if let Some(mut it_node) = item {
            if it_node.get_type() == (&self.ctx, "( Type )").into() {
                let other_tt = it_node.get_edit::<TypeTermEditor>().unwrap();

                if other_tt.write().unwrap().state == State::Ladder {
                    let other = other_tt.read().unwrap().cur_node.get().get_edit::<ListEditor>().unwrap();
                    let buf = other.read().unwrap().data.clone();

                    subladder_list_edit.write().unwrap().up();
                    subladder_list_edit.write().unwrap().up();
                    subladder_list_node.send_cmd_obj(
                        ListCmd::DeleteNexd.into_repr_tree( &self.ctx )
                    );

                    if subladder_list_edit.read().unwrap().get_cursor_warp().tree_addr.len() > 0 {
                        if subladder_list_edit.read().unwrap().get_cursor_warp().tree_addr[0] == -1 {
                            subladder_list_edit.write().unwrap().delete_nexd();
                        }
                    }

                    let l = buf.len();
                    for i in 0..l {
                        subladder_list_edit.write().unwrap().insert( buf.get(i) );
                    }
                    subladder_list_node.dn();
                }
            }
        }
    }

    /* in insert mode, morph the previous element into a ladder and continue there
     */
    pub fn previous_item_into_ladder(&mut self) {
        let app_edit = self.cur_node.get().get_edit::<ListEditor>().expect("editor");
        let mut app_edit = app_edit.write().unwrap();

        let cur = app_edit.get_cursor();

        if cur.tree_addr.len() <= 2 && cur.tree_addr.len() > 0 {
            if cur.tree_addr.len() == 2 {
                app_edit.delete_nexd();
            }

            app_edit.goto(TreeCursor{
                tree_addr: vec![ cur.tree_addr[0]-1 ],
                leaf_mode: ListCursorMode::Select
            });
            
           if let Some(item_node) = app_edit.get_item() {
                let item_typterm = item_node.get_edit::<TypeTermEditor>().expect("typetermedit");
                let mut item_typterm = item_typterm.write().unwrap();
                match item_typterm.state {

                    // if item at cursor is Ladder
                    State::Ladder => {
                        drop(item_typterm);

                        app_edit.dn();
                        app_edit.qnexd();
                    }
                    _ => {
                        item_typterm.goto(TreeCursor::none());
                        drop(item_typterm);
                        
                        // else create new ladder
                        let mut list_node = Context::make_node( &self.ctx, (&self.ctx, "( List Type )").into(), 0 ).unwrap();
                        list_node = list_node.morph( (&self.ctx, "( Type::Ladder )").into() );

                        let mut new_node = TypeTermEditor::with_node(
                            self.ctx.clone(),
                            0,
                            list_node,
                            State::Ladder
                        );

                        // insert old node and split
                        new_node.goto(TreeCursor::home());

                        new_node.send_cmd_obj(
                            ReprTree::new_leaf(
                                (&self.ctx, "( NestedNode )"),
                                SingletonBuffer::<NestedNode>::new( item_node ).get_port().into()
                            )
                        );                        

                        *app_edit.get_item_mut().unwrap().write().unwrap() = new_node;
                        app_edit.dn();
                    }
                }
            }
        }
    }

    /* replace with new ladder node with self as first element
     */
    pub fn morph_to_ladder(&mut self) {
        eprintln!("morph into ladder");
        let old_node = self.cur_node.get().clone();

        /* create a new NestedNode with TerminaltypeEditor,
         * that has same state & child-node as current node.
         */
        let mut old_edit_node = TypeTermEditor::new_node( self.ctx.clone(), 0 );
        let mut old_edit_clone = old_edit_node.get_edit::<TypeTermEditor>().unwrap();
        old_edit_clone.write().unwrap().set_state( self.state );
        old_edit_clone.write().unwrap().close_char.set( old_node.close_char.get() );
        old_edit_clone.write().unwrap().cur_node.set( old_node );

        /* create new list-edit node for the ladder
         */
        let mut new_node = Context::make_node( &self.ctx, (&self.ctx, "( List Type )").into(), 0 ).unwrap();
        new_node = new_node.morph( (&self.ctx, "( Type::Ladder )").into() );

        /* reconfigure current node to display new_node list-editor
         */
        self.close_char.set(new_node.close_char.get());
        self.cur_node.set(new_node);
        self.state = State::Ladder;

        /* insert old node and split
         */
        self.goto(TreeCursor::home());
        self.send_cmd_obj(
            ReprTree::new_leaf(
                (&self.ctx, "( NestedNode )"),
                SingletonBuffer::new( old_edit_node ).get_port().into()
            )
        );

        self.set_addr(0);
        self.dn();
    }
}

