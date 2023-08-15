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
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            let buf = r3vi::buffer::vec::VecBuffer::<char>::new();
                            let data = ReprTree::new_leaf(
                                ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                                buf.get_port().into()
                            );

                            Some(
                                NestedNode::new(ctx, data, depth)
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
    let ctx0 = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ListEditor::init_ctx( &ctx0 );

    
    let mut ctx = ctx0.write().unwrap();
// TODO:: CharEditor::init_ctx( &ctx );
    ctx.add_node_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(CharEditor::new_node(ctx))
            }
        )
    );

    ctx.add_list_typename("Seq".into());
    ctx.add_list_typename("Sequence".into());
    ctx.add_list_typename("SepSeq".into());
    ctx.add_typename("NestedNode".into());


    ctx.add_list_typename("Symbol".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("Symbol").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, None );
                PTYListStyle::for_node( &mut node, ("","","") );

                Some(node)
            }
        )
    );

    ctx.add_node_ctor(
        "Symbol", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    (&ctx, "( List Char )").into(),
                    depth+1
                ).expect("nested node");

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

    ctx.add_list_typename("String".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("String").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, Some('\"') );
                PTYListStyle::for_node( &mut node, ("\"","","\"") );
                Some(node)                
            }
        )
    );

   ctx.add_node_ctor(
        "String", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::App(vec![
                        TypeTerm::TypeID(ctx.read().unwrap().get_typeid("List").unwrap()),
                        TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                    ]),
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );
/*
    ctx.add_list_typename("TypeTerm".into());
    ctx.add_node_ctor(
        "TypeTerm", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Some(TypeTermEditor::new(ctx, depth).into_node(depth))
            }
        )
    );
*/
    ctx.add_typename("TerminalEvent".into());

    drop(ctx);
    ctx0
}

pub fn init_math_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx0 = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    let mut ctx = ctx0.write().unwrap();
    ctx.add_typename("MachineInt".into());
    ctx.add_typename("u32".into());
    ctx.add_typename("u64".into());
    ctx.add_typename("LittleEndian".into());
    ctx.add_typename("BigEndian".into());

    ctx.add_node_ctor(
        "Digit", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
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

    ctx.add_list_typename("PosInt".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("PosInt").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, dst_type| {
                let depth = node.depth.get();
                let editor = node.editor.get().unwrap().downcast::<RwLock<ListEditor>>().unwrap();

                // todo: check src_type parameter to be ( Digit radix )

                match dst_type {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
                                TypeTerm::Num(_radix) => {
                                    PTYListController::for_node(
                                        &mut node,
                                        Some(','),
                                        None,
                                    );

                                    PTYListStyle::for_node(
                                        &mut node,
                                        ("0d", "", "")
                                    );

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

    ctx.add_node_ctor(
        "PosInt", Arc::new(
            |ctx0: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                match dst_typ.clone() {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
                                TypeTerm::Num(radix) => {
                                    let ctx = ctx0.read().unwrap();
                                    let mut node = Context::make_node(
                                        &ctx0,
                                        TypeTerm::App(vec![
                                            TypeTerm::TypeID(ctx.get_typeid("List").unwrap()),
                                            TypeTerm::TypeID(
                                                ctx.get_typeid("Digit").unwrap()
                                            )
                                                .num_arg(radix)
                                                .clone()
                                                .into()
                                        ]),
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
    
    ctx.add_typename("Date".into());
    ctx.add_typename("ISO-8601".into());
    ctx.add_typename("TimeSinceEpoch".into());
    ctx.add_typename("Duration".into());
    ctx.add_typename("Seconds".into());
    ctx.add_typename("ℕ".into());

    drop(ctx);
    ctx0
}

