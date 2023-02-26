use {
    r3vi::{
        view::{
            OuterViewPort
        }
    },
    crate::{
        type_system::{TypeLadder, Context},
        terminal::{
            TerminalStyle, TerminalView,
            make_label
        },
        utils::color::{bg_style_from_depth, fg_style_from_depth},
        tree::{NestedNode, TreeNav}
    },
    std::{sync::{Arc, RwLock}},
};

#[derive(Clone)]
pub enum ProductEditorSegment {
    T( String, usize ),
    N {
        t: TypeLadder,
        editor: Option<NestedNode>,
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
                    move |_i, x|
                    x.add_style_back(fg_style_from_depth(depth)).add_style_back(TerminalStyle::italic(true))
                }
            ),

            ProductEditorSegment::N { t: _, editor: Some(e), ed_depth, cur_depth: _, cur_dist } =>
                e.get_view()
                .map_item({
                    let e = e.clone();
                    let d = *ed_depth;
                    let cur_dist = *cur_dist;

                    move |_i, x| {
                        let c = e.get_cursor();
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
                make_label(&ctx.read().unwrap().type_term_to_str(&t.0[0]))
                .map_item({
                    let _cur_depth = *cur_depth;
                    let _ed_depth = *ed_depth;
                    let cur_dist = *cur_dist;

                    move |_i, x|
                    x.add_style_back(TerminalStyle::fg_color((215,140,95)))
                        .add_style_back(bg_style_from_depth(if cur_dist == 0 { 0 } else { usize::MAX }))
                        .add_style_back(TerminalStyle::bold(cur_dist == 0))
                })
        }        
    }
}

