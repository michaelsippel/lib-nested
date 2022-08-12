use {
    crate::{
        core::{OuterViewPort, TypeLadder, Context},
        terminal::{
            TerminalEditor, TerminalStyle, TerminalView,
            make_label
        },
        tree_nav::{TerminalTreeEditor},
        color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::{sync::{Arc, RwLock}, ops::{Deref, DerefMut}},
    termion::event::{Event, Key},
};

#[derive(Clone)]
pub enum ProductEditorSegment {
    T( String, usize ),
    N {
        t: TypeLadder,
        editor: Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>>,
        cur_depth: usize
    }
}

impl ProductEditorSegment {
    pub fn get_view(&self, ctx: Arc<RwLock<Context>>) -> OuterViewPort<dyn TerminalView> {
        match self {
            ProductEditorSegment::T(t, depth) =>
                make_label(t.as_str())
                .map_item({
                    let depth = *depth;
                    move |i, x|
                    x.add_style_back(fg_style_from_depth(depth))
                }
            ),

            ProductEditorSegment::N { t: _, editor: Some(e), cur_depth } =>
                e.read().unwrap()
                .get_term_view()
                .map_item({
                    let e = e.clone();
                    move |i, x| {
                        let cur_depth = e.read().unwrap().get_cursor().tree_addr.len();
                        x
                            .add_style_back(fg_style_from_depth(cur_depth))//fg_color((250,210,0)))
                            .add_style_back(bg_style_from_depth(cur_depth))
                    }
                }),

            ProductEditorSegment::N{ t, editor: None, cur_depth } =>
                make_label(&ctx.read().unwrap().type_term_to_str(&t[0]))
                .map_item({
                    let cur_depth = 0;
                    move |i, x| x
                            .add_style_back(TerminalStyle::fg_color((130,90,40)))
                            .add_style_back(bg_style_from_depth(cur_depth))
                })
        }        
    }
}

