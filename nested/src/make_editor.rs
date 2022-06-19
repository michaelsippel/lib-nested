
use {
    crate::{
        core::{TypeLadder, Context},
        terminal::{TerminalView},
        tree_nav::{TerminalTreeEditor},
        integer::PosIntEditor,
        list::{ListEditor, PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        product::editor::ProductEditor,
        char_editor::CharEditor
    },
    cgmath::Vector2,
    std::sync::{Arc, RwLock},
};

pub fn make_editor(ctx: Arc<RwLock<Context>>, t: &TypeLadder, depth: usize) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>> {
    let c = ctx.read().unwrap();
    if t[0] == c.type_term_from_str("( PosInt 16 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(16))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( PosInt 10 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(10))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( String )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(CharEditor::new()))
                }),
                SeqDecorStyle::DoubleQuote,
                depth
            )
        ))

    } else if t[0] == c.type_term_from_str("( List Char )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(
                    || { Arc::new(RwLock::new(CharEditor::new())) }
                ),
                SeqDecorStyle::Plain,
                depth
            )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List ℕ )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(PosIntEditor::new(16)))
                }),
                SeqDecorStyle::EnumSet,
                depth
            )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Path )").unwrap() {
        let d = depth + 1;
        Arc::new(RwLock::new(PTYListEditor::new(
            Box::new({
                let d= depth +1;
                move || {
                    Arc::new(RwLock::new(PTYListEditor::new(
                        Box::new(|| {
                            Arc::new(RwLock::new(CharEditor::new()))
                        }),
                        SeqDecorStyle::Plain,
                        d
                    )))
            }}),
            SeqDecorStyle::Path,
            depth
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List RGB )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::<dyn TerminalTreeEditor + Send +Sync>::new(
                Box::new({
                    let d = depth+1;
                    let ctx = ctx.clone();
                    move || {
                        make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( RGB )").unwrap() ], d)
                    }
                }),
                SeqDecorStyle::VerticalSexpr,
                depth
            )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( RGB )").unwrap() {
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t("{ r: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(", g: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(", b: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(" }")
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Vec3i )").unwrap() {
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t("{ x: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(", y: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(", z: ")
                             .with_n( vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )
                             .with_t(" }")
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List Term )").unwrap() {
        Arc::new(RwLock::new(
            PTYListEditor::<dyn TerminalTreeEditor + Send + Sync>::new(
                Box::new({
                    let ctx = ctx.clone();
                    move || {
                        make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( Term )").unwrap() ], depth+1)
                    }
                }),
                SeqDecorStyle::Tuple,
                depth
            )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else { // else: term
        Arc::new(RwLock::new(
            ProductEditor::new(depth, ctx.clone())
                .with_n( vec![ c.type_term_from_str("( List Char )").unwrap() ] )
                .with_n( vec![ c.type_term_from_str("( List Term )").unwrap() ] )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>
    }
}


