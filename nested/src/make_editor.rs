use {
    crate::{
        core::{TypeTerm, Context},
        integer::PosIntEditor,
        list::{PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        sum::SumEditor,
        char_editor::CharEditor,
        Nested
    },
    std::sync::{Arc, RwLock},
};

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(
                    Arc::new(RwLock::new(CharEditor::new_node(&ctx)))
                        as Arc<RwLock<dyn Nested + Send + Sync>>)
            }
        )
    );

    ctx.write().unwrap().add_editor_ctor(
        "List", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            // todo: factor style out of type arGS
                            let style = if args.len() > 1 {
                                match args[1] {
                                    TypeTerm::Num(0) => SeqDecorStyle::Plain,
                                    TypeTerm::Num(1) => SeqDecorStyle::HorizontalSexpr,
                                    TypeTerm::Num(2) => SeqDecorStyle::VerticalSexpr,
                                    TypeTerm::Num(3) => SeqDecorStyle::DoubleQuote,
                                    TypeTerm::Num(4) => SeqDecorStyle::Tuple,
                                    TypeTerm::Num(5) => SeqDecorStyle::EnumSet,
                                    TypeTerm::Num(6) => SeqDecorStyle::Path,
                                    _ => SeqDecorStyle::HorizontalSexpr
                                }
                            } else {
                                SeqDecorStyle::HorizontalSexpr
                            };

                            let delim = if args.len() > 1 {
                                match args[1] {
                                    TypeTerm::Num(0) => Some(' '),
                                    TypeTerm::Num(1) => Some(' '),
                                    TypeTerm::Num(2) => Some('\n'),
                                    TypeTerm::Num(3) => None,
                                    TypeTerm::Num(4) => Some(','),
                                    TypeTerm::Num(5) => Some(','),
                                    TypeTerm::Num(6) => Some('/'),
                                    _ => None
                                }
                            }else {
                                None
                            };

                            Some(
                                Arc::new(RwLock::new(PTYListEditor::new(
                                    Box::new({
                                        move || {
                                            Context::make_editor(ctx.clone(), args[0].clone(), depth + 1).unwrap()
                                        }
                                    }),
                                    style,
                                    delim,
                                    depth
                                    )))
                                    as Arc<RwLock<dyn Nested + Send + Sync>>
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

    ctx.write().unwrap().add_editor_ctor(
        "Symbol", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( List Char 0 )").unwrap(),
                    depth
                )
            }
        )
    );

    ctx.write().unwrap().add_editor_ctor(
        "String", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( List Char 3 )").unwrap(),
                    depth
                )
            }
        )
    );
    
    ctx.write().unwrap().add_editor_ctor(
        "TypeTerm", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                let mut s = SumEditor::new(
                    vec![
                        Context::make_editor(ctx.clone(), ctx.read().unwrap().type_term_from_str("( Symbol )").unwrap(), depth+1).unwrap(),
                        Context::make_editor(ctx.clone(), ctx.read().unwrap().type_term_from_str("( PosInt 10 )").unwrap(), depth+1).unwrap(),
                        Context::make_editor(ctx.clone(), ctx.read().unwrap().type_term_from_str("( List TypeTerm )").unwrap(), depth+1).unwrap(),
                    ]
                );
                s.select(0);
                Some(
                    Arc::new(RwLock::new(
                        s
                    ))
                )
            }
        )
    );

    ctx.write().unwrap().add_typename("TerminalEvent".into());    
    ctx
}

pub fn init_math_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("MachineInt".into());
    ctx.write().unwrap().add_typename("Digit".into());
    ctx.write().unwrap().add_typename("BigEndian".into());
    ctx.write().unwrap().add_editor_ctor(
        "PosInt", Arc::new(
            |_ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {
                                    Some(
                                        Arc::new(RwLock::new(PosIntEditor::new(radix as u32)))
                                            as Arc<RwLock<dyn Nested + Send + Sync>>
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

    ctx
}

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "PathSegment", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( List Char 0 )").unwrap(),
                    depth
                )
            }
        )
    );

    ctx.write().unwrap().add_editor_ctor(
        "Path", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    ctx.clone(),
                    ctx.read().unwrap().type_term_from_str("( List PathSegment 6 )").unwrap(),
                    depth+1
                )
            }
        )
    );

    ctx
}


