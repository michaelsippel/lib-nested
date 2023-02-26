use {
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::{
            char::*,
            list::*,
            integer::*,
            product::*
        },
        tree::{NestedNode},
        diagnostics::{Diagnostics},
        type_system::{MorphismTypePattern},
    },
    std::sync::{Arc, RwLock},
    cgmath::Point2
};

pub fn init_mem_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_node_ctor(
        "Vec", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            let buf = r3vi::buffer::vec::VecBuffer::<char>::new();
                            let data = ReprTree::new_leaf(
                                ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                                buf.get_port().into()
                            );

                            Some(
                                NestedNode::new(depth)
                                    .set_ctx(ctx)
                                    .set_data(data)
                                    .set_editor(Arc::new(RwLock::new(buf)))
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

    ctx
}

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_node_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(CharEditor::new_node(ctx))
            }
        )
    );
    ctx.write().unwrap().add_list_typename("Seq".into());
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
                            let editor = PTYListEditor::new(
                                ctx,
                                args[0].clone(),
                                Some(','),
                                depth + 1
                            );
                            let view = editor.pty_view(("{",", ", "}"));

                            Some(editor
                                    .into_node()
                                    .set_view(view))
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
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("Symbol").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    None,
                    depth
                );

                node.view = Some(pty_editor.pty_view(("", "", "")));
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
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

    ctx.write().unwrap().add_list_typename("String".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("String").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                let pty_editor = PTYListEditor::from_editor(
                    editor,
                    None,
                    depth
                );

                node.view = Some(pty_editor.pty_view((
                    "\"".into(),
                    "".into(),
                    "\"".into()
                )));
                node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                Some(node)                
            }
        )
    );

    ctx.write().unwrap().add_node_ctor(
        "String", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::Type {
                        id: ctx.read().unwrap().get_typeid("List").unwrap(),
                        args: vec![
                            TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                        ]
                    },
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );
/*
    ctx.write().unwrap().add_list_typename("TypeTerm".into());
    ctx.write().unwrap().add_node_ctor(
        "TypeTerm", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Some(TypeTermEditor::new(ctx, depth).into_node(depth))
            }
        )
    );
*/
    ctx.write().unwrap().add_typename("TerminalEvent".into());
    ctx
}

pub fn init_math_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("MachineInt".into());
    ctx.write().unwrap().add_typename("u32".into());
    ctx.write().unwrap().add_typename("LittleEndian".into());
    ctx.write().unwrap().add_typename("BigEndian".into());

    ctx.write().unwrap().add_node_ctor(
        "Digit", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {
                                    let node = DigitEditor::new(ctx.clone(), radix as u32).into_node(depth);                                    
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
    let pattern = MorphismTypePattern {
        src_tyid: ctx.read().unwrap().get_typeid("List"),
        dst_tyid: ctx.read().unwrap().get_typeid("PosInt").unwrap()
    };
    ctx.write().unwrap().add_morphism(pattern,
        Arc::new(
            |mut node, dst_type| {
                let depth = node.depth;
                let editor = node.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();

                // todo: check src_type parameter to be ( Digit radix )
                
                match dst_type {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(_radix) => {
                                    let pty_editor = PTYListEditor::from_editor(
                                        editor,
                                        Some(','),
                                        depth
                                    );

                                    // todo: set data view
                                    node.view = Some(pty_editor.pty_view(
                                        (
                                            "{".into(),
                                            ", ".into(),
                                            "}".into()
                                        )
                                    ));
                                    node.cmd = Some(Arc::new(RwLock::new(pty_editor)));
                                    Some(node)
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

    ctx.write().unwrap().add_node_ctor(
        "PosInt", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                match dst_typ.clone() {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {

                                    let mut node = Context::make_node(
                                        &ctx,
                                        TypeTerm::Type {
                                            id: ctx.read().unwrap().get_typeid("List").unwrap(),
                                            args: vec![
                                                TypeTerm::new(ctx.read().unwrap().get_typeid("Digit").unwrap())
                                                    .num_arg(radix)
                                                    .clone()
                                            ]
                                        },
                                        depth+1
                                    ).unwrap();

                                    node = node.morph(dst_typ);

                                    Some(node)
                                }
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
                        ].into())
                    .with_t(Point2::new(0, 1), "g: ")
                    .with_n(Point2::new(1, 1),
                        vec![
                        ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap()
                        ].into())
                    .with_t(Point2::new(0, 2), "b: ")
                    .with_n(Point2::new(1, 2),
                        vec![
                        ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap()
                        ].into()
                    );

                let view = editor.get_term_view();
                let diag = editor.get_msg_port();
                let editor = Arc::new(RwLock::new(editor));

                let node = NestedNode::new(depth)
                    .set_ctx(ctx)
                    .set_cmd(editor.clone())
                    .set_nav(editor.clone())
                    .set_view(view)
                    .set_diag(diag)
                    ;

                Some(node)
            }
        ));

    ctx.write().unwrap().add_typename("Date".into());
    ctx.write().unwrap().add_typename("ISO-8601".into());
    ctx.write().unwrap().add_typename("TimeSinceEpoch".into());
    ctx.write().unwrap().add_typename("Duration".into());
    ctx.write().unwrap().add_typename("Seconds".into());
    ctx.write().unwrap().add_typename("ℕ".into());
    
    ctx
}

