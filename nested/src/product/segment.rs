use {
    crate::{
        core::{OuterViewPort, TypeLadder, Context},
        terminal::{
            TerminalEditor, TerminalStyle, TerminalView,
            make_label
        },
        list::{ListCursorMode},
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
        ed_depth: usize,
        cur_depth: usize,
        cur_dist: isize
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
                    x.add_style_back(fg_style_from_depth(depth)).add_style_back(TerminalStyle::italic(true))
                }
            ),

            ProductEditorSegment::N { t: _, editor: Some(e), ed_depth, cur_depth, cur_dist } =>
                e.read().unwrap()
                .get_term_view()
                .map_item({
                    let e = e.clone();
                    let d = *ed_depth;
                    let cur_dist = *cur_dist;

                    move |i, x| {
                        let c = e.read().unwrap().get_cursor();
                        let cur_depth = c.tree_addr.len();
                        let select =
                            if cur_dist == 0 {
                                cur_depth
                            } else {
                                usize::MAX
                            };

                        return x
                            .add_style_back(bg_style_from_depth(select))
                            .add_style_back(TerminalStyle::bold(select==1))
                            .add_style_back(fg_style_from_depth(d));
                    }
                }),

            ProductEditorSegment::N{ t, editor: None, ed_depth, cur_depth, cur_dist } =>
                make_label(&ctx.read().unwrap().type_term_to_str(&t[0]))
                .map_item({
                    let cur_depth = *cur_depth;
                    let ed_depth = *ed_depth;
                    let cur_dist = *cur_dist;

                    move |i, x|
                    x.add_style_back(TerminalStyle::fg_color((215,140,95)))
                        .add_style_back(bg_style_from_depth(if cur_dist == 0 { 0 } else { usize::MAX }))
                        .add_style_back(TerminalStyle::bold(cur_dist == 0))
                })
        }        
    }
}

