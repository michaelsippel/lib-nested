use {
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::{
            char::*,
            list::*,
            integer::*,
            product::*,
            sum::*
        },
        tree::{NestedNode},        
        terminal::{TerminalEditor},
        diagnostics::{Diagnostics},
        type_system::{TypeTermEditor, MorphismTypePattern},
    },
    std::sync::{Arc, RwLock},
    cgmath::Point2
};

pub fn init_mem_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("Vec".into());

    ctx
}

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_node_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(CharEditor::new_node(&ctx))
            }
        )
    );
    ctx.write().unwrap().add_list_typename("Sequence".into());

    ctx.write().unwrap().add_list_typename("List".into());
    ctx.write().unwrap().add_node_ctor(
        "List", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            Some(
                                PTYListEditor::new(
                                    ctx,
                                    args[0].clone(),
                                    ListStyle::HorizontalSexpr,
                                    depth + 1
                                ).into_node()
                            )
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );

    ctx.write().unwrap().add_list_typename("Symbol".into());

    let pattern = MorphismTypePattern {
        src_type: ctx.read().unwrap().type_term_from_str("( List Char )"),
        dst_tyid: ctx.read().unwrap().get_typeid("Symbol").unwrap()
    };

    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, dst_type:_, depth| {
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    ListStyle::Plain,
                    depth
                );

                node.view = Some(pty_editor.pty_view());
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)                
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "Symbol", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                        ]
                    },
                    depth
                ).unwrap();

                node = Context::morph_node(ctx, node, dst_typ);

                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_list_typename("String".into());
    ctx.write().unwrap().add_node_ctor(
        "String", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                let mut node = PTYListEditor::new(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                    ListStyle::DoubleQuote,
                    depth + 1
                ).into_node();

                node.data = Some(ReprTree::ascend(
                    &node.data.unwrap(),
                    ctx.read().unwrap().type_term_from_str("( String )").unwrap()
                ));

                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_list_typename("TypeTerm".into());
    ctx.write().unwrap().add_node_ctor(
        "TypeTerm", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Some(TypeTermEditor::new(ctx, depth).into_node())
            }
        )
    );

    ctx.write().unwrap().add_typename("TerminalEvent".into());
    ctx
}

pub fn init_math_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("MachineInt".into());
    ctx.write().unwrap().add_typename("u32".into());
    ctx.write().unwrap().add_typename("BigEndian".into());

    ctx.write().unwrap().add_node_ctor(
        "Digit", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {
                                    let node = DigitEditor::new(ctx.clone(), radix as u32).into_node();                                    
                                    Some(
                                        node
                                    )
                                },
                                _ => None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );

    ctx.write().unwrap().add_list_typename("PosInt".into());
    ctx.write().unwrap().add_node_ctor(
        "PosInt", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {
                                    Some(
                                        PosIntEditor::new(ctx.clone(), radix as u32).into_node()
                                    )
                                },
                                _ => None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );    

    ctx.write().unwrap().add_list_typename("RGB".into());
    ctx.write().unwrap().add_node_ctor(
        "RGB", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                let editor = ProductEditor::new(depth, ctx.clone())
                    .with_t(Point2::new(0, 0), "r: ")
                    .with_n(Point2::new(1, 0),
                        vec![
                        ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap()
                        ])
                    .with_t(Point2::new(0, 1), "g: ")
                    .with_n(Point2::new(1, 1),
                        vec![
                        ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap()
                        ])
                    .with_t(Point2::new(0, 2), "b: ")
                    .with_n(Point2::new(1, 2),
                        vec![
                        ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap()
                        ]
                    );

                let view = editor.get_term_view();
                let diag = editor.get_msg_port();
                let editor = Arc::new(RwLock::new(editor));

                let node = NestedNode::new()
                    .set_ctx(ctx)
                    .set_cmd(editor.clone())
                    .set_nav(editor.clone())
                    .set_view(view)
                    .set_diag(diag)
                    ;

                Some(node)
            }
        ));
    
    ctx
}

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_list_typename("PathSegment".into());
    ctx.write().unwrap().add_node_ctor(
        "PathSegment", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                let mut node = PTYListEditor::new(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                    ListStyle::Plain,
                    depth + 1
                ).into_node();

                node.data = Some(ReprTree::ascend(
                    &node.data.unwrap(),
                    ctx.read().unwrap().type_term_from_str("( PathSegment )").unwrap()
                ));

                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_list_typename("Path".into());
    ctx.write().unwrap().add_node_ctor(
        "Path", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                let mut node = PTYListEditor::new(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( PathSegment )").unwrap(),
                    ListStyle::Path,
                    depth + 1
                ).into_node();

                node.data = Some(ReprTree::ascend(
                    &node.data.unwrap(),
                    ctx.read().unwrap().type_term_from_str("( Path )").unwrap()
                ));

                Some(node)
            }
        )
    );

    ctx
}


