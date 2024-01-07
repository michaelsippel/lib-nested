use {
    r3vi::{
        view::{
            Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort,
            singleton::*,
            sequence::*,
        },
        projection::projection_helper::*
    },
    crate::{
        editors::list::{ListCursor, ListCursorMode},
        edit_tree::{EditTree}
    },
    std::sync::Arc,
    std::sync::RwLock,
};

pub enum ListSegment {
    InsertCursor,
    Item {
        editor: EditTree,
        cur_dist: isize,
    }
}

pub struct ListSegmentSequence {
    data: Arc<dyn SequenceView<Item = EditTree>>,
    cursor: Arc<dyn SingletonView<Item = ListCursor>>,

    cur_cursor: ListCursor,

    port: ViewPort<dyn SequenceView<Item = ListSegment>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = ListSegment>>>>,
    proj_helper: ProjectionHelper<usize, Self>,
}

impl View for ListSegmentSequence {
    type Msg = usize;
}

impl SequenceView for ListSegmentSequence {
    type Item = ListSegment;

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
        Some(if let Some(cur) = c.idx {
            match c.mode {
                ListCursorMode::Select => {
                    ListSegment::Item {
                        editor: self.data.get(idx)?,
                        cur_dist: cur - *idx as isize
                    }
                }
                ListCursorMode::Insert => {
                    if *idx < cur as usize {
                        ListSegment::Item {
                            editor: self.data.get(idx)?,
                            cur_dist: cur - *idx as isize
                        }
                    } else if *idx == cur as usize {
                        ListSegment::InsertCursor
                    } else {
                        ListSegment::Item {
                            editor: self.data.get(&(*idx - 1))?,
                            cur_dist: cur - *idx as isize
                        }
                    }
                }
            }
        } else {
            ListSegment::Item {
                editor: self.data.get(&idx)?,
                cur_dist: *idx as isize + 1
            }
        })
    }
}

impl ListSegmentSequence {
    pub fn new(
        cursor_port: OuterViewPort<dyn SingletonView<Item = ListCursor>>,
        data_port: OuterViewPort<dyn SequenceView<Item = EditTree>>,
    ) -> Arc<RwLock<Self>> {
        let out_port = ViewPort::new();
        let mut proj_helper = ProjectionHelper::new(out_port.update_hooks.clone());
        let proj = Arc::new(RwLock::new(ListSegmentSequence {
            cur_cursor: cursor_port.get_view().get(),
            port: out_port.clone(),

            cursor: proj_helper.new_singleton_arg(0, cursor_port, |s: &mut Self, _msg| {
                let _old_cursor = s.cur_cursor;
                s.cur_cursor = s.cursor.get();

                // todo: optimize
                s.cast.notify_each(0..=s.data.len().unwrap_or(0) + 1);
            }),

            data: proj_helper.new_sequence_arg(1, data_port, |s: &mut Self, idx| {
                if let Some(cur_idx) = s.cur_cursor.idx {
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
            cast: out_port.inner().get_broadcast(),
            proj_helper,
        }));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.inner().set_view(Some(proj.clone()));

        proj
    }

    pub fn get_view(&self) -> OuterViewPort<dyn SequenceView<Item = ListSegment>> {
        self.port.outer()
    }
}
