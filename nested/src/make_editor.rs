
use {
    crate::{
        core::{TypeTerm, TypeLadder, Context, OuterViewPort},
        terminal::{TerminalView, TerminalEditor, TerminalEvent, TerminalEditorResult, make_label},
        tree::{TreeNav},
        integer::PosIntEditor,
        list::{ListEditor, PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        product::editor::ProductEditor,
        sum::SumEditor,
        char_editor::CharEditor,
        diagnostics::Diagnostics,
        Nested
    },
    cgmath::{Vector2, Point2},
    std::sync::{Arc, RwLock},
};

enum RhsNode {
    Sum (
        Arc<RwLock< PTYListEditor< RhsNode > >>
    ),
    Product (
        Arc<RwLock< PTYListEditor< RhsNode > >>
    ),
    String(
        Arc<RwLock< PTYListEditor< CharEditor > >>
    )
}

impl TreeNav for RhsNode {}

impl TerminalEditor for RhsNode {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        make_label("todo")
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        TerminalEditorResult::Continue
    }
}

impl Diagnostics for RhsNode {}
impl Nested for RhsNode {}

struct GrammarRuleEditor {
    lhs: Arc<RwLock<PTYListEditor<CharEditor>>>,
    rhs: Arc<RwLock<PTYListEditor<RhsNode>>>
}

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let mut ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                Some(
                    Arc::new(RwLock::new(CharEditor::new()))
                        as Arc<RwLock<dyn Nested + Send + Sync>>)
            }
        )
    );

    ctx.write().unwrap().add_editor_ctor(
        "List", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id, args
                    } => {
                        if args.len() > 0 {
                            // todod factor style out of type arGS
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
                            }else {
                                SeqDecorStyle::HorizontalSexpr
                            };

                            let delim = if args.len() > 1 {
                                match args[1] {
                                    TypeTerm::Num(0) => ' ',
                                    TypeTerm::Num(1) => ' ',
                                    TypeTerm::Num(2) => '\n',
                                    TypeTerm::Num(3) => '"',
                                    TypeTerm::Num(4) => ',',
                                    TypeTerm::Num(5) => ',',
                                    TypeTerm::Num(6) => '/',
                                    _ => '\0'
                                }
                            }else {
                                '\0'
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
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
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
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
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
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
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
    
    ctx
}

pub fn init_math_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let mut ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_typename("BigEndian".into());
    ctx.write().unwrap().add_editor_ctor(
        "PosInt", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, _depth: usize| {
                match ty {
                    TypeTerm::Type {
                        id, args
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
    let mut ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_editor_ctor(
        "PathSegment", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
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
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
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


