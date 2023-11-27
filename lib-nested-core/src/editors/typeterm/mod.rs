mod ctx;
mod nav;
mod cmd;

pub use ctx::init_ctx;

use {
    r3vi::{
        view::{OuterViewPort, singleton::*},
        buffer::{singleton::*}
    },
    laddertypes::{TypeID, TypeTerm},
    crate::{
        reprTree::{Context, ReprTree},
        editTree::{NestedNode, TreeNav, TreeNavResult, TreeCursor},
        editors::{list::{ListCursorMode, ListEditor, ListCmd}, ObjCommander},
    },
    std::{sync::{Arc, RwLock}}
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
    depth: OuterViewPort<dyn SingletonView<Item = usize>>,

    buf: SingletonBuffer< TypeTerm >,

    // editing/parsing state
    state: State,

    // child node
    cur_node: SingletonBuffer< NestedNode >
}

impl TypeTermEditor {
    pub fn from_type_term(ctx: Arc<RwLock<Context>>, depth: OuterViewPort<dyn SingletonView<Item = usize>>, term: &TypeTerm) -> NestedNode {
        let mut node = TypeTermEditor::new_node(ctx.clone(), depth.clone());
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
                    let arg_node = TypeTermEditor::from_type_term( parent_ctx.clone(), depth.map(|d| d+1), x );

                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            Context::parse(&ctx, "NestedNode"),
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
                    let arg_node = TypeTermEditor::from_type_term( parent_ctx.clone(), depth.map(|d| d+1), x );

                    node.send_cmd_obj(
                        ReprTree::new_leaf(
                            Context::parse(&ctx, "NestedNode"),
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
        }
 
        node.goto(TreeCursor::none());
        node
    }

    fn set_state(&mut self, new_state: State) {
        let mut node = match new_state {
            State::Any => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Char>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Sym") )
            }
            State::App => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Type>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::App") )
            }
            State::Ladder => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Type>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Ladder") )
            }
            State::AnySymbol => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Char>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Sym") )
            },
            State::FunSymbol => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Char>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Sym::Fun") )
            },
            State::VarSymbol => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "<List Char>"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Sym::Var") )
            }
            State::Num => {
                crate::editors::integer::PosIntEditor::new(self.ctx.clone(), 10)
                    .into_node()
                    .morph( Context::parse(&self.ctx, "Type::Lit::Num") )
            }
            State::Char => {
                Context::make_node( &self.ctx, Context::parse(&self.ctx, "Char"), self.depth.map(|x| x) ).unwrap()
                    .morph( Context::parse(&self.ctx, "Type::Lit::Char") )
            }
        };

        node.goto(TreeCursor::home());

        let _editor = node.editor.get();
        self.close_char.set(node.close_char.get());
        self.cur_node.set(node);
        self.state = new_state;
    }

    pub fn new_node(ctx: Arc<RwLock<Context>>, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> NestedNode {
        let ctx : Arc<RwLock<Context>> = Arc::new(RwLock::new(Context::with_parent(Some(ctx))));
        ctx.write().unwrap().meta_chars.push('~');
        ctx.write().unwrap().meta_chars.push('<');

        let mut symb_node = Context::make_node( &ctx, Context::parse(&ctx, "<List Char>"), depth ).unwrap();
        symb_node = symb_node.morph( Context::parse(&ctx, "Type::Sym") );

        Self::with_node(
            ctx.clone(),
            symb_node,
            State::Any
        )
    }

    fn with_node(ctx: Arc<RwLock<Context>>, cur_node: NestedNode, state: State) -> NestedNode {
        let buf = SingletonBuffer::<TypeTerm>::new( TypeTerm::unit() );

        let data = Arc::new(RwLock::new(ReprTree::new(
            Context::parse(&ctx, "Type")
        )));

        let editor = TypeTermEditor {
            ctx: ctx.clone(),
            data: data.clone(),
            state,
            buf,
            cur_node: SingletonBuffer::new(cur_node.clone()),
            close_char: SingletonBuffer::new(None),
            spillbuf: Arc::new(RwLock::new(Vec::new())),
            depth: cur_node.depth.clone()
        };
/* FIXME
        let view = editor.cur_node
            .get_port()
            .map(|node| {
                node.view.clone().unwrap_or(r3vi::view::ViewPort::new().into_outer())
            })
            .to_grid()
            .flatten();
        */
        let _cc = editor.cur_node.get().close_char;
        let editor = Arc::new(RwLock::new(editor));

        let mut super_node = NestedNode::new(ctx, data, cur_node.depth)
//            .set_view(view)
            .set_nav(editor.clone())
            .set_cmd(editor.clone())
            .set_editor(editor.clone());

        editor.write().unwrap().close_char = super_node.close_char.clone();
        super_node.spillbuf = editor.read().unwrap().spillbuf.clone();

        super_node
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
        let subladder_list_node = self.cur_node.get().clone();
        let subladder_list_edit = subladder_list_node.get_edit::<ListEditor>().unwrap();

        let subladder_list_edit = subladder_list_edit.read().unwrap();
        if subladder_list_edit.data.len() == 0 {
            self.set_state( State::Any );
        }
    }
    
    /* unwrap a ladder if it only contains one element
     */
    pub fn normalize_singleton(&mut self) {
        eprintln!("normalize singleton");

        if self.state == State::Ladder {           
            let subladder_list_node = self.cur_node.get().clone();
            let subladder_list_edit = subladder_list_node.get_edit::<ListEditor>().unwrap();

            let subladder_list_edit = subladder_list_edit.read().unwrap();
            if subladder_list_edit.data.len() == 1 {
                let it_node = subladder_list_edit.data.get(0);
                let it_node = it_node.read().unwrap();
                if it_node.get_type() == Context::parse(&self.ctx, "Type") {
                    let other_tt = it_node.get_edit::<TypeTermEditor>().unwrap();

                    let mut other_tt = other_tt.write().unwrap();

                    other_tt.normalize_singleton();
                    other_tt.depth.0.set_view( self.depth.map(|x| x).get_view() );

                    self.close_char.set(other_tt.close_char.get());
                    self.cur_node.set(other_tt.cur_node.get());
                    self.state = other_tt.state;
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

            // select previous element
            app_edit.goto(TreeCursor{
                tree_addr: vec![ cur.tree_addr[0]-1 ],
                leaf_mode: ListCursorMode::Select
            });

            // get selected element
            if let Some(item_node) = app_edit.get_item() {
                let item_typterm = item_node.get_edit::<TypeTermEditor>().expect("typetermedit");
                let mut item_typterm = item_typterm.write().unwrap();
                if item_typterm.state != State::Ladder {
                    item_typterm.morph_to_list( State::Ladder );
                }

                item_typterm.goto(TreeCursor {
                    tree_addr: vec![ -1 ],
                    leaf_mode: ListCursorMode::Insert
                });
            }
        }
    }
    
    /* replace with new list-node (ladder/app) with self as first element
     */
     pub(super) fn morph_to_list(&mut self, state: State) {
        eprintln!("morph into ladder");

        let mut old_node = self.cur_node.get().clone();

        /* reconfigure current node to display new_node list-editor
         */
        self.set_state( state );

        /* create a new NestedNode with TerminaltypeEditor,
         * that has same state & child-node as current node.
         */
        let old_edit_node = TypeTermEditor::new_node( self.ctx.clone(), SingletonBuffer::new(0).get_port() );
        old_node.depth.0.set_view( old_edit_node.depth.map(|x|x).get_view() );
        
        let old_edit_clone = old_edit_node.get_edit::<TypeTermEditor>().unwrap();
        old_edit_clone.write().unwrap().set_state( self.state );
        old_edit_clone.write().unwrap().cur_node.set( old_node );

        /* insert old node and split
         */
        self.goto(TreeCursor::home());
        self.send_child_cmd(
            ReprTree::new_leaf(
                Context::parse(&self.ctx, "NestedNode"),
                SingletonBuffer::new( old_edit_node ).get_port().into()
            )
        );
    }
}

