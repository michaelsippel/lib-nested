use {
    std::sync::Arc,
    std::sync::RwLock,
    crate::{
        core::{
            View,
            InnerViewPort,
            OuterViewPort,
            Observer,
            ObserverExt,
            ObserverBroadcast
        },
        projection::ProjectionHelper,

        singleton::SingletonView,
        sequence::SequenceView,
        terminal::TerminalView,

        list::{ListCursor, ListCursorMode}
    }
};

pub enum ListEditorViewSegment {
    InsertCursor,
    View(OuterViewPort<dyn TerminalView>),
    Select(OuterViewPort<dyn TerminalView>),
    Modify(OuterViewPort<dyn TerminalView>)
}

pub struct ListEditorView {
    cursor: Arc<dyn SingletonView<Item = ListCursor>>,
    data: Arc<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
    cur_cursor: ListCursor,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = ListEditorViewSegment>>>>,
    proj_helper: ProjectionHelper<usize, Self>
}

impl View for ListEditorView {
    type Msg = usize;
}

impl SequenceView for ListEditorView {
    type Item = ListEditorViewSegment;

    fn len(&self) -> Option<usize> {
        match self.cursor.get().mode {
            ListCursorMode::Insert => Some(self.data.len()? + if self.cur_cursor.idx.is_some() { 1 } else { 0 }),
            _ => self.data.len()
        }
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        Some(
            if let Some(cur) = self.cur_cursor.idx {
                match self.cur_cursor.mode {
                    ListCursorMode::Select => {
                        if *idx == cur {
                            ListEditorViewSegment::Select(self.data.get(&idx)?)
                        } else {
                            ListEditorViewSegment::View(self.data.get(&idx)?)
                        }
                    }
                    ListCursorMode::Insert => {
                        if *idx < cur {
                            ListEditorViewSegment::View(self.data.get(&idx)?)
                        } else if *idx == cur {
                            ListEditorViewSegment::InsertCursor
                        } else {
                            ListEditorViewSegment::View(self.data.get(&(idx-1))?)
                        }
                    }
                    ListCursorMode::Modify => {
                        if *idx == cur {
                            ListEditorViewSegment::Modify(self.data.get(&idx)?)
                        } else {
                            ListEditorViewSegment::View(self.data.get(&idx)?)
                        }
                    }
                }
            } else {
                ListEditorViewSegment::View(self.data.get(&idx)?)
            }
        )
    }
}

impl ListEditorView {
    pub fn new(
        cursor_port: OuterViewPort<dyn SingletonView<Item = ListCursor>>,
        data_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = ListEditorViewSegment>>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(
                ListEditorView {
                    cur_cursor: ListCursor::default(),
                    cursor: proj_helper.new_singleton_arg(
                        0,
                        cursor_port,
                        |s: &mut Self, _msg| {
                            let old_cursor = s.cur_cursor;
                            let new_cursor = s.cursor.get();
                            s.cur_cursor = new_cursor;

                            s.cast.notify_each(
                                0 ..= s.data.len().unwrap_or(0)+1
                            );
                        }),
                    data: proj_helper.new_sequence_arg(
                        1,
                        data_port,
                        |s: &mut Self, idx| {
                            if let Some(cur_idx) = s.cur_cursor.idx {
                                match s.cur_cursor.mode {
                                    ListCursorMode::Insert => {
                                        if *idx < cur_idx {
                                            s.cast.notify(idx);
                                        } else {
                                            s.cast.notify(&(*idx + 1));
                                        }
                                    },
                                    _ => {
                                        s.cast.notify(idx);
                                    }
                                }
                            } else {
                                s.cast.notify(idx);
                            }
                        }),
                    cast: out_port.get_broadcast(),
                    proj_helper
                }
            ));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.set_view(Some(proj.clone()));

        proj
    }
}

