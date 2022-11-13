
use {
    crate::{
        core::{TypeLadder, Context, OuterViewPort},
        terminal::{TerminalView, TerminalEditor, TerminalEvent, TerminalEditorResult, make_label},
        tree::{TreeNav},
        integer::PosIntEditor,
        list::{ListEditor, PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        product::editor::ProductEditor,
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

pub fn make_editor(ctx: Arc<RwLock<Context>>, t: &TypeLadder, depth: usize) -> Arc<RwLock<dyn Nested + Send + Sync>> {
    let c = ctx.read().unwrap();
    if t[0] == c.type_term_from_str("( PosInt 16 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(16))) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( PosInt 10 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(10))) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( String )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(CharEditor::new()))
                }),
                SeqDecorStyle::DoubleQuote,
                '"',
                depth
            )
        ))

    } else if t[0] == c.type_term_from_str("( Symbol )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(CharEditor::new()))
                }),
                SeqDecorStyle::Plain,
                ' ',
                depth
            )
        ))

    } else if t[0] == c.type_term_from_str("( List String )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new({
                    let d = depth + 1;
                    let ctx = ctx.clone();
                    move || {
                        make_editor(
                            ctx.clone(),
                            &vec![ctx.read().unwrap().type_term_from_str("( String )").unwrap()],
                            d
                        )
                    }
                }),
                SeqDecorStyle::EnumSet,
                '"',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>
    } else if t[0] == c.type_term_from_str("( List Symbol )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new({
                    let d = depth + 1;
                    let ctx = ctx.clone();
                    move || {
                        make_editor(
                            ctx.clone(),
                            &vec![ctx.read().unwrap().type_term_from_str("( Symbol )").unwrap()],
                            d
                        )
                    }
                }),
                SeqDecorStyle::EnumSet,
                ' ',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List Char )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(
                    || { Arc::new(RwLock::new(CharEditor::new())) }
                ),
                SeqDecorStyle::Plain,
                '\n',
                depth+1
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List ℕ )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(PosIntEditor::new(16)))
                }),
                SeqDecorStyle::EnumSet,
                ',',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Path )").unwrap() {
        Arc::new(RwLock::new(PTYListEditor::new(
            Box::new({
                let d= depth+1;
                move || {
                    Arc::new(RwLock::new(PTYListEditor::new(
                        Box::new(|| {
                            Arc::new(RwLock::new(CharEditor::new()))
                        }),
                        SeqDecorStyle::Plain,
                        '\n',
                        d
                    )))
            }}),
            SeqDecorStyle::Path,
            '/',
            depth
        ))) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List Path )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new({
                    let d = depth + 1;
                    let ctx = ctx.clone();
                    move || {
                        make_editor(
                            ctx.clone(),
                            &vec![ctx.read().unwrap().type_term_from_str("( Path )").unwrap()],
                            d
                        )
                    }
                }),
                SeqDecorStyle::EnumSet,
                ',',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List RGB )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::<dyn Nested + Send +Sync>::new(
                {
                    let d = depth+1;
                    let ctx = ctx.clone();
                    Box::new(move || {
                        make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( RGB )").unwrap() ], d)
                    })
                },
                SeqDecorStyle::VerticalSexpr,
                ',',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( RGB )").unwrap() {
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(0, 0), "{   ")
                             .with_t(Point2::new(1, 1), "r: ")
                             .with_n(Point2::new(2, 1), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(1, 2), "g: ")
                             .with_n(Point2::new(2, 2), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(1, 3), "b: ")
                             .with_n(Point2::new(2, 3), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(0, 4), "}   ")
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Vec3i )").unwrap() {
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(0, 0), "{")
                             .with_t(Point2::new(1, 1), "x: ")
                             .with_n(Point2::new(2, 1), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(1, 2), "y: ")
                             .with_n(Point2::new(2, 2), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(1, 3), "z: ")
                             .with_n(Point2::new(2, 3), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(0, 4), "}")
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Json )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::<dyn Nested + Send + Sync>::new(
                Box::new({
                    let ctx = ctx.clone();
                    move || {
                        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( String )").unwrap() ] )
                                             .with_t(Point2::new(1, 0), ": ")
                                             .with_n(Point2::new(2, 0), vec![ ctx.read().unwrap().type_term_from_str("( Json )").unwrap() ] )
                        )) as Arc<RwLock<dyn Nested + Send + Sync>>
                    }
                }),
                SeqDecorStyle::VerticalSexpr,
                '\n',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>
            
    } else if t[0] == c.type_term_from_str("( List Term )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::<dyn Nested + Send + Sync>::new(
                Box::new({
                    let ctx = ctx.clone();
                    move || {
                        make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( Term )").unwrap() ], depth+1)
                    }
                }),
                SeqDecorStyle::Tuple,
                '\n',
                depth
            )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>

    } else { // else: term
        Arc::new(RwLock::new(
            PTYListEditor::new(
                || {
                    Arc::new(RwLock::new(CharEditor::new()))
                },
                SeqDecorStyle::DoubleQuote,
                ' ',
                depth
            )
        ))
    }
}


