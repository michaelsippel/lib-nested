
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


pub fn init_ctx() -> Arc<RwLock<Context>> {
        let mut ctx = Arc::new(RwLock::new(Context::new()));
    for tn in vec![
        "MachineWord", "MachineInt", "MachineSyllab", "Bits",
        "Vec", "Stream", "Json",
        "Sequence", "AsciiString", "UTF-8-String", "Char", "String", "Symbol",
        "PosInt", "Digit", "LittleEndian", "BigEndian",
        "DiffStream", "ℕ", "List", "PathSegment", "Path", "Term", "RGB", "Vec3i"
    ] { ctx.write().unwrap().add_typename(tn.into()); }

    ctx.write().unwrap().add_editor_ctor(
        "Char", Box::new(
            |ctx: &Context, ty: TypeTerm, _depth: usize| {
                Some(
                    Arc::new(RwLock::new(CharEditor::new()))
                        as Arc<RwLock<dyn Nested + Send + Sync>>)
            }
        )
    );
    ctx.write().unwrap().add_editor_ctor(
        "Symbol", Box::new(
            |ctx: &Context, ty: TypeTerm, depth: usize| {
                ctx.make_editor(
                    ctx.type_term_from_str("( List Char 0 )").unwrap(),
                    depth
                )
            }
        )
    );
    ctx.write().unwrap().add_editor_ctor(
        "String", Box::new(
            |ctx: &Context, ty: TypeTerm, depth: usize| {
                ctx.make_editor(
                    ctx.type_term_from_str("( List Char 3 )").unwrap(),
                    depth
                )
            }
        )
    );
    ctx.write().unwrap().add_editor_ctor(
        "PosInt", Box::new(
            |ctx: &Context, ty: TypeTerm, _depth: usize| {
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
    
    ctx.write().unwrap().add_editor_ctor(
        "List", Box::new({
            let ctx = ctx.clone();
            move |c_: &Context, ty: TypeTerm, depth: usize| {
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
                                        let ctx = ctx.clone();
                                        move || {
                                            ctx.read().unwrap().make_editor(args[0].clone(), depth + 1).unwrap()
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
        }
    ));

    ctx.write().unwrap().add_editor_ctor(
        "RGB", Box::new({
            let c = ctx.clone();
            move |ctx: &Context, ty: TypeTerm, depth: usize| {
                Some(Arc::new(RwLock::new(ProductEditor::new(depth, c.clone())
                                          .with_t(Point2::new(0, 0), "{   ")
                                          .with_t(Point2::new(1, 1), "r: ")
                                          .with_n(Point2::new(2, 1), vec![ ctx.type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                                          .with_t(Point2::new(1, 2), "g: ")
                                          .with_n(Point2::new(2, 2), vec![ ctx.type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                                          .with_t(Point2::new(1, 3), "b: ")
                                          .with_n(Point2::new(2, 3), vec![ ctx.type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                                          .with_t(Point2::new(0, 4), "}   ")
                )) as Arc<RwLock<dyn Nested + Send + Sync>>)
            }
        }));

    ctx.write().unwrap().add_editor_ctor(
        "PathSegment", Box::new(
            |ctx: &Context, ty: TypeTerm, depth: usize| {
                ctx.make_editor(
                    ctx.type_term_from_str("( List Char 0 )").unwrap(),
                    depth
                )
            }
        )
    );
    ctx.write().unwrap().add_editor_ctor(
        "Path", Box::new(
            |ctx: &Context, ty: TypeTerm, depth: usize| {
                ctx.make_editor(
                    ctx.type_term_from_str("( List PathSegment 6 )").unwrap(),
                    depth+1
                )
            }
        )
    );

    ctx.write().unwrap().add_editor_ctor(
        "Term", Box::new(
            |ctx: &Context, ty: TypeTerm, depth: usize| {
                let mut s = SumEditor::new(
                    vec![
                        ctx.make_editor(ctx.type_term_from_str("( Symbol )").unwrap(), depth+1).unwrap(),
                        ctx.make_editor(ctx.type_term_from_str("( PosInt 10 )").unwrap(), depth+1).unwrap(),
                        ctx.make_editor(ctx.type_term_from_str("( List Term )").unwrap(), depth+1).unwrap(),
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



