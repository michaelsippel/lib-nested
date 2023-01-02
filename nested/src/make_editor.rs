use {
    crate::{
        type_system::{TypeTerm, Context},
        integer::{DigitEditor, PosIntEditor},
        list::{PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        sum::SumEditor,
        char_editor::CharEditor,
        type_term_editor::TypeTermEditor,
        Nested
    },
    std::sync::{Arc, RwLock},
};

pub fn init_mem_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("Vec".into());

    ctx
}

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(CharEditor::new_node(&ctx))
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
                                    TypeTerm::Num(0) => None,
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
                                PTYListEditor::new(
                                    ctx.clone(),
                                    args[0].clone(),
                                    style,
                                    delim,
                                    depth
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

    ctx.write().unwrap().add_editor_ctor(
        "Symbol", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    &ctx,
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
                    &ctx,
                    ctx.read().unwrap().type_term_from_str("( List Char 3 )").unwrap(),
                    depth
                )
            }
        )
    );
    
    ctx.write().unwrap().add_editor_ctor(
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
    ctx.write().unwrap().add_typename("BigEndian".into());

    //ctx.write().unwrap().add_typename("Digit".into());

    ctx.write().unwrap().add_editor_ctor(
        "Digit", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id: _, args
                    } => {
                        if args.len() > 0 {
                            match args[0] {
                                TypeTerm::Num(radix) => {
                                    Some(
                                        DigitEditor::new(ctx.clone(), radix as u32).into_node()
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
    
    ctx.write().unwrap().add_editor_ctor(
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

    ctx
}

pub fn init_os_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "PathSegment", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Context::make_editor(
                    &ctx,
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
                    &ctx,
                    ctx.read().unwrap().type_term_from_str("( List PathSegment 6 )").unwrap(),
                    depth+1
                )
            }
        )
    );

    ctx
}


