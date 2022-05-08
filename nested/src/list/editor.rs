use {
    crate::{
        core::{OuterViewPort, ViewPort},
        list::{
            editor_view::{ListEditorView, ListEditorViewSegment},
            ListCursor, ListCursorMode, ListDecoration, SExprView,
        },
        sequence::SequenceView,
        singleton::{SingletonBuffer, SingletonView},
        terminal::{
            make_label, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree_nav::{TerminalTreeEditor, TreeCursor, TreeNav, TreeNavResult},
        vec::VecBuffer,
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

#[derive(Clone, Copy)]
pub enum ListEditorStyle {
    HorizontalSexpr,
    VerticalSexpr,
    Tuple(usize),
    Path,
    String,
    Clist,
    Hex,
    Plain,
}

pub struct ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>,
{
    cursor: SingletonBuffer<ListCursor>,
    data: VecBuffer<Arc<RwLock<ItemEditor>>>,

    cursor_port: ViewPort<dyn SingletonView<Item = ListCursor>>,
    data_port: ViewPort<RwLock<Vec<Arc<RwLock<ItemEditor>>>>>,

    make_item_editor: FnMakeItemEditor,

    style: ListEditorStyle,
    _level: usize,
    cur_dist: Arc<RwLock<usize>>,
}

impl<ItemEditor, FnMakeItemEditor> TreeNav for ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>,
{
    fn get_cursor(&self) -> TreeCursor {
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert | ListCursorMode::Select => TreeCursor {
                leaf_mode: cur.mode,
                tree_addr: if let Some(i) = cur.idx {
                    vec![i]
                } else {
                    vec![]
                },
            },
            ListCursorMode::Modify => {
                if let Some(i) = cur.idx {
                    if i < self.data.len() {
                        let mut sub_cur = self.data.get(i).read().unwrap().get_cursor();
                        sub_cur.tree_addr.insert(0, i);
                        return sub_cur;
                    }
                }
                TreeCursor {
                    leaf_mode: cur.mode,
                    tree_addr: vec![],
                }
            }
        }
    }

    fn goto(&mut self, new_cur: TreeCursor) -> TreeNavResult {
        let old_cur = self.cursor.get();
        if old_cur.mode == ListCursorMode::Modify {
            if let Some(i) = old_cur.idx {
                let ce = self.data.get_mut(i);
                let mut cur_edit = ce.write().unwrap();
                cur_edit.goto(TreeCursor::default());
            }
        }

        if new_cur.tree_addr.len() == 1 {
            self.cursor.set(ListCursor {
                mode: new_cur.leaf_mode,
                idx: Some(new_cur.tree_addr[0]),
            });
            TreeNavResult::Continue
        } else if new_cur.tree_addr.len() > 1 && new_cur.tree_addr[0] < self.data.len() {
            self.cursor.set(ListCursor {
                mode: ListCursorMode::Modify,
                idx: Some(new_cur.tree_addr[0]),
            });

            let ne = self.data.get_mut(new_cur.tree_addr[0]);
            let mut nxt_edit = ne.write().unwrap();

            nxt_edit.goto(TreeCursor {
                leaf_mode: new_cur.leaf_mode,
                tree_addr: new_cur.tree_addr[1..].iter().cloned().collect(),
            });

            TreeNavResult::Continue
        } else {
            self.cursor.set(ListCursor {
                mode: new_cur.leaf_mode,
                idx: None,
            });
            TreeNavResult::Continue
        }
    }

    fn goto_end(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        let i = cur.idx.unwrap_or(0);

        if self.data.len() == 0 && cur.idx.is_none() {
            self.cursor.set(ListCursor {
                mode: ListCursorMode::Insert,
                idx: Some(0),
            });
            return TreeNavResult::Continue;
        }

        if i < self.data.len() {
            match cur.mode {
                ListCursorMode::Insert => {
                    if i < self.data.len() || cur.idx.is_none() {
                        cur.idx = Some(self.data.len());
                        self.cursor.set(cur);
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::default());
                        TreeNavResult::Exit
                    }
                }
                ListCursorMode::Select => {
                    if self.data.len() == 0 && cur.idx.is_none() {
                        self.cursor.set(ListCursor {
                            mode: ListCursorMode::Insert,
                            idx: Some(0),
                        });
                        return TreeNavResult::Continue;
                    }

                    if i + 1 < self.data.len() || cur.idx.is_none() {
                        cur.idx = Some(self.data.len() - 1);
                        self.cursor.set(cur);
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::default());
                        TreeNavResult::Exit
                    }
                }
                ListCursorMode::Modify => {
                    let ce = self.data.get_mut(i);
                    let mut cur_edit = ce.write().unwrap();
                    let cur_mode = cur_edit.get_cursor().leaf_mode;
                    let depth = cur_edit.get_cursor().tree_addr.len();
                    match cur_edit.goto_end() {
                        TreeNavResult::Continue => TreeNavResult::Continue,
                        TreeNavResult::Exit => {
                            drop(cur_edit);

                            self.up();

                            if i + 1 < self.data.len() {
                                self.set_mode(ListCursorMode::Select);
                                self.nexd();

                                for _x in 1..depth {
                                    self.dn();
                                    self.goto_home();
                                }

                                self.set_leaf_mode(cur_mode);
                                self.dn();
                                self.goto_end();

                                return TreeNavResult::Continue;
                            }

                            self.cursor.set(ListCursor::default());
                            TreeNavResult::Exit
                        }
                    }
                }
            }
        } else {
            self.cursor.set(ListCursor::default());
            TreeNavResult::Exit
        }
    }

    fn goto_home(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        if self.data.len() == 0 && cur.idx.is_none() {
            self.cursor.set(ListCursor {
                mode: ListCursorMode::Insert,
                idx: Some(0),
            });
            return TreeNavResult::Continue;
        }

        match cur.mode {
            ListCursorMode::Insert | ListCursorMode::Select => {
                if cur.idx != Some(0) {
                    self.cursor.set(ListCursor {
                        mode: if self.data.len() == 0 {
                            ListCursorMode::Insert
                        } else {
                            cur.mode
                        },
                        idx: Some(0),
                    });
                    TreeNavResult::Continue
                } else {
                    self.cursor.set(ListCursor::default());
                    TreeNavResult::Exit
                }
            }
            ListCursorMode::Modify => {
                if let Some(ce) = self.get_item() {
                    let mut cur_edit = ce.write().unwrap();
                    let cur_mode = cur_edit.get_cursor().leaf_mode;
                    let depth = cur_edit.get_cursor().tree_addr.len();

                    match cur_edit.goto_home() {
                        TreeNavResult::Exit => {
                            drop(cur_edit);

                            if let Some(i) = cur.idx {
                                if i > 0 {
                                    self.set_mode(ListCursorMode::Select);
                                    self.pxev();

                                    for _x in 1..depth {
                                        self.dn();
                                        self.goto_end();
                                    }

                                    self.dn();
                                    self.set_leaf_mode(cur_mode);
                                    //self.goto_home();
                                    return TreeNavResult::Continue;
                                }
                            }
                        }
                        TreeNavResult::Continue => { return TreeNavResult::Continue; }
                    };
                }

                self.cursor.set(ListCursor::default());
                TreeNavResult::Exit
            }
        }
    }

    fn up(&mut self) -> TreeNavResult {
        let cur = self.cursor.get();
        if cur.mode == ListCursorMode::Modify {
            if let Some(i) = cur.idx {
                let ce = self.data.get_mut(i);
                let mut cur_edit = ce.write().unwrap();
                let mode = cur_edit.get_cursor().leaf_mode;

                match cur_edit.up() {
                    TreeNavResult::Exit => {
                        self.set_mode(mode);
                    }
                    TreeNavResult::Continue => {}
                }

                TreeNavResult::Continue
            } else {
                TreeNavResult::Exit
            }
        } else {
            self.cursor.set(ListCursor {
                mode: cur.mode,
                idx: None,
            });
            TreeNavResult::Exit
        }
    }

    fn dn(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();

        if cur.idx.is_none() {
            self.goto_home()
        } else {
            match cur.mode {
                ListCursorMode::Insert | ListCursorMode::Select => {
                    if let Some(i) = cur.idx {
                        if i < self.data.len() {
                            self.set_mode(ListCursorMode::Modify);
                            self.data.get_mut(i).write().unwrap().goto(TreeCursor {
                                leaf_mode: cur.mode,
                                tree_addr: vec![],
                            });
                            self.data.get_mut(i).write().unwrap().dn();
                            *self.cur_dist.write().unwrap() += 1;
                        }
                    }
                    TreeNavResult::Continue
                }
                ListCursorMode::Modify => self.get_item().unwrap().write().unwrap().dn(),
            }
        }
    }

    fn pxev(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        if let Some(i) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert | ListCursorMode::Select => {
                    if i > 0 {
                        cur.idx = Some(i - 1);
                        self.cursor.set(cur);
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::default());
                        TreeNavResult::Exit
                    }
                }
                ListCursorMode::Modify => {
                    let ce = self.get_item().unwrap();
                    let mut cur_edit = ce.write().unwrap();

                    let cur_mode = cur_edit.get_cursor().leaf_mode;
                    let depth = cur_edit.get_cursor().tree_addr.len();

                    match cur_edit.pxev() {
                        TreeNavResult::Exit => {
                            drop(cur_edit);
                            self.up();

                            if i > 0 {
                                self.set_mode(ListCursorMode::Select);
                                self.pxev();

                                for _x in 1..depth {
                                    self.dn();
                                    self.goto_end();
                                }

                                self.set_leaf_mode(cur_mode);
                                self.dn();
                                self.goto_end();
                                TreeNavResult::Continue
                            } else {
                                TreeNavResult::Exit
                            }
                        }
                        TreeNavResult::Continue => TreeNavResult::Continue,
                    }
                }
            }
        } else {
            TreeNavResult::Exit
        }
    }

    fn nexd(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        if let Some(i) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => {
                    if i < self.data.len() {
                        cur.idx = Some(i + 1);
                        self.cursor.set(cur);
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::default());
                        TreeNavResult::Exit
                    }
                }
                ListCursorMode::Select => {
                    if i + 1 < self.data.len() {
                        cur.idx = Some(i + 1);
                        self.cursor.set(cur);
                        TreeNavResult::Continue
                    } else {
                        self.cursor.set(ListCursor::default());
                        TreeNavResult::Exit
                    }
                }
                ListCursorMode::Modify => {
                    let ce = self.data.get(i);
                    let mut cur_edit = ce.write().unwrap();

                    let depth = cur_edit.get_cursor().tree_addr.len();
                    let cur_mode = cur_edit.get_cursor().leaf_mode;

                    match cur_edit.nexd() {
                        TreeNavResult::Exit => {
                            drop(cur_edit);
                            drop(ce);
                            //self.up();

                            if i + 1 < self.data.len() {
                                self.set_mode(ListCursorMode::Select);
                                self.nexd();

                                for _x in 1..depth {
                                    self.dn();
                                }

                                self.set_leaf_mode(cur_mode);
                                self.dn();

                                TreeNavResult::Continue
                            } else {
                                self.cursor.set(ListCursor::default());
                                TreeNavResult::Exit
                            }
                        }
                        TreeNavResult::Continue => TreeNavResult::Continue,
                    }
                }
            }
        } else {
            TreeNavResult::Exit
        }
    }
}

impl<ItemEditor, FnMakeItemEditor> TerminalEditor for ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>,
{
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        match self.style {
            ListEditorStyle::HorizontalSexpr => self.horizontal_sexpr_view(),
            ListEditorStyle::VerticalSexpr => self.vertical_sexpr_view(),
            ListEditorStyle::Tuple(depth) => self.tuple_view(depth),
            ListEditorStyle::Path => self.path_view(),
            ListEditorStyle::String => self.string_view(),
            ListEditorStyle::Clist => self.clist_view(),
            ListEditorStyle::Hex => self.hex_view(),
            ListEditorStyle::Plain => self.plain_view(),
        }
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        let mut cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => match event {
                    TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                        if idx > 0 && idx <= self.data.len() {
                            cur.idx = Some(idx - 1);
                            self.cursor.set(cur);
                            self.data.remove(idx - 1);
                            TerminalEditorResult::Continue
                        } else {
                            TerminalEditorResult::Exit
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Delete)) => {
                        if idx < self.data.len() {
                            self.data.remove(idx);
                            TerminalEditorResult::Continue
                        } else {
                            TerminalEditorResult::Exit
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                    | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                        self.set_mode(ListCursorMode::Select);
                        TerminalEditorResult::Continue
                    }
                    _ => {
                        let new_edit = (self.make_item_editor)();
                        self.data.insert(idx, new_edit.clone());
                        self.set_mode(ListCursorMode::Modify);
                        let mut ne = new_edit.write().unwrap();

                        match ne.handle_terminal_event(event) {
                            TerminalEditorResult::Exit => {
                                self.cursor.set(ListCursor {
                                    mode: ListCursorMode::Insert,
                                    idx: Some(idx + 1),
                                });
                            }
                            _ => {}
                        }
                        TerminalEditorResult::Continue
                    }
                },
                ListCursorMode::Select => match event {
                    TerminalEvent::Input(Event::Key(Key::Char('\t')))
                    | TerminalEvent::Input(Event::Key(Key::Insert)) => {
                        self.set_mode(ListCursorMode::Insert);
                        TerminalEditorResult::Continue
                    }
                    TerminalEvent::Input(Event::Key(Key::Delete)) => {
                        self.data.remove(idx);

                        if self.data.len() == 0 {
                            self.cursor.set(ListCursor::default());
                        } else if idx == self.data.len() {
                            self.cursor.set(ListCursor {
                                mode: ListCursorMode::Select,
                                idx: Some(idx - 1),
                            });
                        }
                        TerminalEditorResult::Continue
                    }
                    _ => TerminalEditorResult::Continue,
                },
                ListCursorMode::Modify => {
                    let ce = self.data.get_mut(idx);
                    let mut cur_edit = ce.write().unwrap();

                    match cur_edit.handle_terminal_event(event) {
                        TerminalEditorResult::Exit => {
                            cur_edit.up();
                            drop(cur_edit);
                            drop(ce);

                            match event {
                                TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                    // todo: join instead of remove
                                    self.cursor.set(ListCursor {
                                        mode: ListCursorMode::Insert,
                                        idx: Some(idx),
                                    });

                                    self.data.remove(idx);
                                }
                                _ => {
                                    // todo: split

                                    self.cursor.set(ListCursor {
                                        mode: ListCursorMode::Insert,
                                        idx: Some(idx + 1),
                                    });
                                }
                            }
                        }
                        TerminalEditorResult::Continue => {}
                    }

                    TerminalEditorResult::Continue
                }
            }
        } else {
            TerminalEditorResult::Continue
        }
    }
}

impl<ItemEditor, FnMakeItemEditor> TerminalTreeEditor for ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>> + Send + Sync,
{}

impl<ItemEditor, FnMakeItemEditor> ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>,
{
    pub fn get_seg_seq_view(
        &self,
    ) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let segment_view_port = ViewPort::<dyn SequenceView<Item = ListEditorViewSegment>>::new();
        ListEditorView::new(
            self.cursor_port.outer(),
            self.data_port
                .outer()
                .to_sequence()
                .map(|ed| ed.read().unwrap().get_term_view()),
            segment_view_port.inner(),
        );

        segment_view_port.into_outer().map(move |segment| {
            let cursor_col = (90, 60, 200);
            match segment {
                ListEditorViewSegment::InsertCursor => {
                    make_label("|").map_item(move |_pt, atom| {
                        atom.add_style_back(TerminalStyle::fg_color(cursor_col))
                            .add_style_back(TerminalStyle::bold(true))
                    })
                }
                ListEditorViewSegment::Select(sub_view) => sub_view.map_item(move |_pt, atom| {
                    let old_col = atom.style.bg_color.unwrap_or(cursor_col);
                    atom.add_style_front(TerminalStyle::bg_color((
                        (old_col.0 as f32 * 0.4) as u8,
                        (old_col.1 as f32 * 0.4) as u8,
                        (old_col.2 as f32 * 0.4) as u8,
                    )))
                }),
                ListEditorViewSegment::Modify(sub_view) => {
                    sub_view.clone().map_item(
                        move |_pt, atom| {
                            let old_col = atom.style.bg_color.unwrap_or(cursor_col);
                            atom.add_style_front(TerminalStyle::bg_color((
                                (old_col.0 as f32 * 0.4) as u8,
                                (old_col.1 as f32 * 0.4) as u8,
                                (old_col.2 as f32 * 0.4) as u8,
                            )))
                        }, //.add_style_back(TerminalStyle::bold(true))
                    )
                }
                ListEditorViewSegment::View(sub_view) => sub_view.clone(),
            }
        })
    }

    pub fn horizontal_sexpr_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view().horizontal_sexpr_view(1)
    }

    pub fn vertical_sexpr_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view().vertical_sexpr_view(0)
    }

    pub fn path_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("<", ">", "/", 0)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn string_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("\"", "\"", "", 1)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn clist_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("{", "}", ", ", 1)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn tuple_view(&self, depth: usize) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("(", ")", ", ", depth)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn hex_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("0x", "", "", 0)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn plain_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view().to_grid_horizontal().flatten()
    }

    pub fn new(make_item_editor: FnMakeItemEditor, style: ListEditorStyle) -> Self {
        let cursor_port = ViewPort::new();
        let data_port = ViewPort::new();
        let cursor = SingletonBuffer::new(ListCursor::default(), cursor_port.inner());
        let data = VecBuffer::<Arc<RwLock<ItemEditor>>>::new(data_port.inner());

        let mut le = ListEditor {
            data,
            data_port,
            cursor,
            cursor_port,

            style,
            make_item_editor,
            _level: 0,
            cur_dist: Arc::new(RwLock::new(0)),
        };
        le.set_style(style);
        le
    }

    pub fn set_style(&mut self, style: ListEditorStyle) {
        self.style = style;
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>> {
        self.data_port.outer().to_sequence()
    }

    pub fn get_list_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursor>> {
        self.cursor_port.outer()
    }

    pub fn get_item(&self) -> Option<Arc<RwLock<ItemEditor>>> {
        if let Some(idx) = self.cursor.get().idx {
            if idx < self.data.len() {
                Some(self.data.get(idx))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    fn set_idx(&mut self, idx: isize) {
        let cur = self.cursor.get();
        let mode = cur.mode;

        if idx < 0 {
            self.cursor.set(ListCursor {
                mode,
                idx: Some((self.data.len() as isize + idx) as usize),
            });
        } else {
            self.cursor.set(ListCursor {
                mode,
                idx: Some(idx as usize),
            });
        }
    }

    fn set_mode(&mut self, mode: ListCursorMode) {
        let mut cur = self.cursor.get();

        if cur.mode == ListCursorMode::Insert && mode != ListCursorMode::Insert {
            if let Some(idx) = cur.idx {
                if idx == self.data.len() && idx > 0 {
                    cur.idx = Some(idx - 1);
                }
            }
        }

        cur.mode = mode;

        self.cursor.set(cur);
    }
}

impl<ItemEditor, FnMakeItemEditor> ListEditor<ItemEditor, FnMakeItemEditor>
where
    ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
    FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>,
{
    fn set_leaf_mode(&mut self, mode: ListCursorMode) {
        let mut c = self.get_cursor();
        c.leaf_mode = mode;
        self.goto(c);
    }
}

