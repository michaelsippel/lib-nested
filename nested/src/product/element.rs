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
        list::{ListEditor, ListCursorMode, ListEditorStyle}
    },
    cgmath::{Point2, Vector2},
    std::{sync::{Arc, RwLock}, ops::{Deref, DerefMut}},
    termion::event::{Event, Key},
};


#[derive(Clone)]
pub enum ProductEditorElement {
    T( String ),
    N {
        t: TypeLadder,
        editor: Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>>,
        select: bool
    }
}

impl ProductEditorElement {
    pub fn get_view(&self, ctx: Arc<RwLock<Context>>) -> OuterViewPort<dyn TerminalView> {
        match self {
            ProductEditorElement::T(t) =>
                make_label(t.as_str())
                .map_item(
                    |i, x|
                    x.add_style_back(TerminalStyle::fg_color((0,120,200)))
                ),

            ProductEditorElement::N {t: _, editor: Some(e), select} =>
                e.read().unwrap()
                .get_term_view()
                .map_item({ let select = *select;
                            move |i, x| x
                            .add_style_back(TerminalStyle::fg_color((250,210,0)))
                            .add_style_back(
                                if select {
                                    TerminalStyle::bg_color((40,40,40))
                                } else {
                                    TerminalStyle::default()
                                }
                            )
                }),

            ProductEditorElement::N{t, editor: None, select} =>
                make_label(&ctx.read().unwrap().type_term_to_str(&t[0]))
                .map_item({ let select = *select;
                            move |i, x| x
                            .add_style_back(TerminalStyle::fg_color((130,90,40)))
                            .add_style_back(
                                if select {
                                    TerminalStyle::bg_color((40,40,40))
                                } else {
                                    TerminalStyle::default()
                                }
                            )
                })
        }        
    }
}

