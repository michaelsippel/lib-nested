use {
    crate::{
        core::{InnerViewPort, Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View},
        list::{ListCursor, ListCursorMode},
        projection::ProjectionHelper,
        sequence::SequenceView,
        singleton::SingletonView,
        terminal::{TerminalView, TerminalStyle, make_label},
        tree_nav::TerminalTreeEditor,
        color::{bg_style_from_depth, fg_style_from_depth}
    },
    std::sync::Arc,
    std::sync::RwLock,
};

pub enum ListSegment<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    InsertCursor,
    Item {
        editor: Arc<RwLock<ItemEditor>>,
        depth: usize,
        cur_dist: isize,
    }
}

pub trait PTYSegment {
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView>;
}

impl<ItemEditor> PTYSegment for ListSegment<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        match self {
            ListSegment::InsertCursor => {
                make_label("|")
                    .map_item(move |_pt, atom| {
                     atom.add_style_front(TerminalStyle::fg_color((150,80,230)))
                        .add_style_front(TerminalStyle::bold(true))
                    })
            }
            ListSegment::Item{ editor, depth, cur_dist } => {
                let e = editor.clone();
                let d = *depth;
                let cur_dist = *cur_dist;
                editor.read().unwrap().get_term_view().map_item(move |_pt, atom| {
                    let c = e.read().unwrap().get_cursor();
                    let cur_depth = c.tree_addr.len();
                    let select =
                        if cur_dist == 0 {
                            cur_depth
                        } else {
                            usize::MAX
                        };
                    atom
                        .add_style_back(bg_style_from_depth(select))
                        .add_style_back(TerminalStyle::bold(select==1))
                        .add_style_back(fg_style_from_depth(d))
                })
            }
        }
    }
}

pub struct ListSegmentSequence<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    data: Arc<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>>,
    cursor: Arc<dyn SingletonView<Item = ListCursor>>,

    depth: usize,
    cur_cursor: ListCursor,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = ListSegment<ItemEditor>>>>>,
    proj_helper: ProjectionHelper<usize, Self>,
}

impl<ItemEditor> View for ListSegmentSequence<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    type Msg = usize;
}

impl<ItemEditor> SequenceView for ListSegmentSequence<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    type Item = ListSegment<ItemEditor>;

    fn len(&self) -> Option<usize> {
        match self.cur_cursor.mode {
            ListCursorMode::Insert => {
                Some(self.data.len()? + if self.cur_cursor.idx.is_some() { 1 } else { 0 })
            }
            _ => self.data.len(),
        }
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let c = self.cursor.get();
        Some(if let Some(mut cur) = c.idx {
            match c.mode {
                ListCursorMode::Select => {
                    ListSegment::Item {
                        editor: self.data.get(idx)?,
                        depth: self.depth,
                        cur_dist: cur - *idx as isize
                    }
                }
                ListCursorMode::Insert => {
                    if *idx < cur as usize {
                        ListSegment::Item {
                            editor: self.data.get(idx)?,
                            depth: self.depth,
                            cur_dist: cur - *idx as isize
                        }
                    } else if *idx == cur as usize {
                        ListSegment::InsertCursor
                    } else {
                        ListSegment::Item {
                            editor: self.data.get(&(*idx - 1))?,
                            depth: self.depth,
                            cur_dist: cur - *idx as isize
                        }
                    }
                }
            }
        } else {
            ListSegment::Item {
                editor: self.data.get(&idx)?,
                depth: self.depth,
                cur_dist: *idx as isize + 1
            }
        })
    }
}

impl<ItemEditor> ListSegmentSequence<ItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static
{
    pub fn new(
        cursor_port: OuterViewPort<dyn SingletonView<Item = ListCursor>>,
        data_port: OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = ListSegment<ItemEditor>>>,
        depth: usize
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(ListSegmentSequence {
            cur_cursor: cursor_port.get_view().get(),
            depth,

            cursor: proj_helper.new_singleton_arg(0, cursor_port, |s: &mut Self, _msg| {
                let _old_cursor = s.cur_cursor;
                s.cur_cursor = s.cursor.get();

                // todo: optimize
                s.cast.notify_each(0..=s.data.len().unwrap_or(0) + 1);
            }),

            data: proj_helper.new_sequence_arg(1, data_port, |s: &mut Self, idx| {
                if let Some(mut cur_idx) = s.cur_cursor.idx {
                    match s.cur_cursor.mode {
                        ListCursorMode::Insert => {
                            if *idx < cur_idx as usize {
                                s.cast.notify(idx);
                            } else {
                                s.cast.notify(&(*idx + 1));
                            }
                        }
                        _ => {
                            s.cast.notify(idx);
                        }
                    }
                } else {
                    s.cast.notify(idx);
                }
            }),
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.set_view(Some(proj.clone()));

        proj
    }
}
