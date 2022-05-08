
use {
    crate::{
        core::{ViewPort, OuterViewPort, Observer, port::UpdateTask, TypeTerm, TypeLadder, Context},
        terminal::{
            Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView,
            make_label
        },
        sequence::{SequenceView},
        tree_nav::{TreeNav, TerminalTreeEditor, TreeCursor, TreeNavResult},
        vec::{VecBuffer, MutableVecAccess},
        index::buffer::IndexBuffer,
        integer::PosIntEditor,
        string_editor::{StringEditor, CharEditor},
        list::{ListEditor, ListCursorMode, ListEditorStyle},
        product::editor::ProductEditor
    },
    cgmath::{Point2, Vector2},
    std::{sync::{Arc, RwLock}, ops::{Deref, DerefMut}},
    termion::event::{Event, Key},
};

pub fn make_editor(ctx: Arc<RwLock<Context>>, t: &TypeLadder, depth: usize) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>> {
    let c = ctx.read().unwrap();
    if t[0] == c.type_term_from_str("( PosInt 16 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(16))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( PosInt 10 BigEndian )").unwrap() {
        Arc::new(RwLock::new(PosIntEditor::new(10))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( String )").unwrap() {
        Arc::new(RwLock::new(StringEditor::new())) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List Char )").unwrap() {
        Arc::new(RwLock::new(ListEditor::new(
            || { Arc::new(RwLock::new(CharEditor::new())) },
            ListEditorStyle::Plain
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List ℕ )").unwrap() {
        Arc::new(RwLock::new(ListEditor::new(
            || {
                Arc::new(RwLock::new(PosIntEditor::new(16)))
            },
            ListEditorStyle::HorizontalSexpr
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( Path )").unwrap() {
        Arc::new(RwLock::new(ListEditor::new(
            || {
                Arc::new(RwLock::new(ListEditor::new(
                    || {
                        Arc::new(RwLock::new(CharEditor::new()))
                    },
                    ListEditorStyle::Plain
                )))
            },
            ListEditorStyle::Path
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else if t[0] == c.type_term_from_str("( List RGB )").unwrap() {
        Arc::new(RwLock::new(ListEditor::new({
            let ctx = ctx.clone();
            move || {
                make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( RGB )").unwrap() ], depth+1)
            }
        },
            ListEditorStyle::VerticalSexpr
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

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
        Arc::new(RwLock::new(ListEditor::new({
            let ctx = ctx.clone();
            move || {
                make_editor(ctx.clone(), &vec![ ctx.read().unwrap().type_term_from_str("( Term )").unwrap() ], depth+1)
            }
        },
            ListEditorStyle::Tuple(depth)
        ))) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>

    } else { // else: term
        Arc::new(RwLock::new(
            ProductEditor::new(depth, ctx.clone())
                .with_n( vec![ c.type_term_from_str("( List Char )").unwrap() ] )
                .with_n( vec![ c.type_term_from_str("( List Term )").unwrap() ] )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>
    }
}


