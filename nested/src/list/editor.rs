use {
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    crate::{
        core::{
            View,
            ViewPort,
            OuterViewPort,
            InnerViewPort,
            ObserverBroadcast,
            Observer,
            ObserverExt,
            context::{
                ReprTree,
                Object,
                MorphismType,
                MorphismMode,
                Context
            }
        },
        projection::ProjectionHelper,
        singleton::{SingletonView, SingletonBuffer},
        sequence::{SequenceView},
        vec::{VecBuffer},
        terminal::{
            TerminalView,
            TerminalStyle,
            TerminalEvent,
            TerminalEditor,
            TerminalEditorResult,
            make_label
        },
        leveled_term_view::LeveledTermView,
        list::{SExprView, ListDecoration, ListCursor, ListCursorMode},
        tree_nav::{TreeCursor, TreeNav, TreeNavResult, TerminalTreeEditor}
    }
};

pub struct ListEditor<ItemEditor, FnMakeItemEditor>
where ItemEditor: TerminalEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>
{
    cursor: SingletonBuffer<ListCursor>,
    data: VecBuffer<Arc<RwLock<ItemEditor>>>,
    data_sequence_port: OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>>,
    make_item_editor: FnMakeItemEditor,
    level: usize,
    segment_seq: OuterViewPort<dyn SequenceView<Item = ListEditorViewSegment>>,

    terminal_view: OuterViewPort<dyn TerminalView>
}

impl<ItemEditor, FnMakeItemEditor> TreeNav for ListEditor<ItemEditor, FnMakeItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>
{
    fn get_cursor(&self) -> TreeCursor {
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert |
            ListCursorMode::Select => {
                TreeCursor {
                    leaf_mode: cur.mode,
                    tree_addr: if let Some(i) = cur.idx { vec![ i ] } else { vec![] }
                }
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
                    tree_addr: vec![]
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
            self.cursor.set(ListCursor{
                mode: new_cur.leaf_mode,
                idx: Some(new_cur.tree_addr[0])
            });
            TreeNavResult::Continue
        } else if new_cur.tree_addr.len() > 1 && new_cur.tree_addr[0] < self.data.len() {
            self.cursor.set(ListCursor {
                mode: ListCursorMode::Modify,
                idx: Some(new_cur.tree_addr[0])
            });

            let ne = self.data.get_mut(new_cur.tree_addr[0]);
            let mut nxt_edit = ne.write().unwrap();

            nxt_edit.goto(
                TreeCursor {
                    leaf_mode: new_cur.leaf_mode,
                    tree_addr: new_cur.tree_addr[1..].iter().cloned().collect()
                });

            TreeNavResult::Continue
        } else {
            self.cursor.set(ListCursor {
                mode: new_cur.leaf_mode,
                idx: None
            });
            TreeNavResult::Exit
        }
    }

    fn goto_end(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        let i = cur.idx.unwrap_or(0);
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
                    if i+1 < self.data.len() || cur.idx.is_none() {
                        cur.idx = Some(self.data.len()-1);
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

                    match cur_edit.goto_end() {
                        TreeNavResult::Continue => {
                            TreeNavResult::Continue
                        }
                        TreeNavResult::Exit => {
                            if let Some(i) = cur.idx {
                                if i+1 < self.data.len() {
                                    cur_edit.up();
                                    drop(cur_edit);

                                    self.nexd();
                                    self.set_mode(ListCursorMode::Modify);

                                    let ne = self.get_item().unwrap();
                                    let mut nxd_edit = ne.write().unwrap();

                                    nxd_edit.goto(
                                        TreeCursor {
                                            leaf_mode: cur_mode,
                                            tree_addr: vec![]
                                        }
                                    );
                                    nxd_edit.goto_end();

                                    TreeNavResult::Continue
                                } else {
                                    self.cursor.set(ListCursor::default());
                                    TreeNavResult::Exit
                                }
                            } else {
                                self.cursor.set(ListCursor::default());
                                TreeNavResult::Exit
                            }
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
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert |
            ListCursorMode::Select => {
                if cur.idx != Some(0) {
                    self.cursor.set(
                        ListCursor {
                            mode: cur.mode,
                            idx: Some(0)
                        }
                    );
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

                match cur_edit.goto_home() {
                    TreeNavResult::Exit => {
                        if let Some(i) = cur.idx {
                            if i > 0 {
                                cur_edit.up();
                                drop(cur_edit);

                                self.pxev();
                                self.set_mode(ListCursorMode::Modify);

                                let pe = self.get_item().unwrap();
                                let mut pxv_edit = pe.write().unwrap();

                                pxv_edit.goto(
                                    TreeCursor {
                                        leaf_mode: cur_mode,
                                        tree_addr: vec![]
                                    }
                                );
                                pxv_edit.goto_home();

                                TreeNavResult::Continue
                            } else {
                                self.cursor.set(ListCursor::default());
                                TreeNavResult::Exit
                            }
                        } else {
                            self.cursor.set(ListCursor::default());
                            TreeNavResult::Exit
                        }
                    }
                    TreeNavResult::Continue => TreeNavResult::Continue
                }
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

                return TreeNavResult::Continue;
            }
        }

        self.cursor.set(ListCursor {
            mode: cur.mode,
            idx: None
        });
        TreeNavResult::Exit
    }

    fn dn(&mut self) -> TreeNavResult {
        let cur = self.cursor.get();
        match cur.mode {
            ListCursorMode::Insert |
            ListCursorMode::Select => {
                if let Some(i) = cur.idx {
                    if i < self.data.len() {
                        self.set_mode(ListCursorMode::Modify);
                        self.data.get_mut(i).write().unwrap().goto(
                            TreeCursor {
                                leaf_mode: cur.mode,
                                tree_addr: vec![ 0 ]
                            }
                        );
                    }
                }
            }
            ListCursorMode::Modify => {
                self.get_item().unwrap().write().unwrap().dn();
            }
        }
        TreeNavResult::Continue
    }

    fn pxev(&mut self) -> TreeNavResult {
        let mut cur = self.cursor.get();
        if let Some(i) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert |
                ListCursorMode::Select => {
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

                    match cur_edit.pxev() {
                        TreeNavResult::Exit => {
                            drop(cur_edit);
                            if i > 0 {
                                self.up();
                                self.pxev();
                                self.set_mode(ListCursorMode::Modify);

                                let pe = self.get_item().unwrap();                                
                                let mut pxv_edit = pe.write().unwrap();
                                pxv_edit.goto(
                                    TreeCursor {
                                        leaf_mode: cur_mode,
                                        tree_addr: vec![]
                                    }
                                );
                                pxv_edit.goto_end();

                                TreeNavResult::Continue
                            } else {
                                self.cursor.set(ListCursor::default());
                                TreeNavResult::Exit
                            }
                        },
                        TreeNavResult::Continue => TreeNavResult::Continue
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
                    if i+1 < self.data.len() {
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
                    let cur_mode = cur_edit.get_cursor().leaf_mode;

                    match cur_edit.nexd() {
                        TreeNavResult::Exit => {
                            if i+1 < self.data.len() {
                                cur_edit.up();
                                drop(cur_edit);
                                drop(ce);

                                self.set_mode(cur_mode);
                                self.nexd();
                                self.dn();
                                TreeNavResult::Continue
                            } else {
                                self.cursor.set(ListCursor::default());
                                TreeNavResult::Exit
                            }
                        }
                        TreeNavResult::Continue => TreeNavResult::Continue
                    }
                }
            }
        } else {
            TreeNavResult::Exit
        }
    }
}

impl<ItemEditor, FnMakeItemEditor> TerminalEditor for ListEditor<ItemEditor, FnMakeItemEditor>
where ItemEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>
{
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.terminal_view.clone()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        let mut cur = self.cursor.get();
        if let Some(idx) = cur.idx {
            match cur.mode {
                ListCursorMode::Insert => {
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                            if idx > 0 {
                                self.data.remove(idx-1);
                                cur.idx = Some(idx-1);
                                self.cursor.set(cur);
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
                        TerminalEvent::Input(Event::Key(Key::Char('\t'))) |
                        TerminalEvent::Input(Event::Key(Key::Insert)) => {
                            self.set_mode(ListCursorMode::Select);
                            TerminalEditorResult::Continue
                        }
                        _ => {
                            let new_edit = (self.make_item_editor)();
                            self.data.insert(idx, new_edit.clone());
                            self.dn();

                            match new_edit.write().unwrap().handle_terminal_event(event) {
                                TerminalEditorResult::Exit => {
                                    self.cursor.set(ListCursor {
                                        mode: ListCursorMode::Insert,
                                        idx: Some(idx+1)
                                    });
                                }
                                _ => {}
                            }
                            TerminalEditorResult::Continue
                        }
                    }
                }
                ListCursorMode::Select => {
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Char('\t'))) |
                        TerminalEvent::Input(Event::Key(Key::Insert)) => {
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
                                    idx: Some(idx-1)
                                });
                            }
                            TerminalEditorResult::Continue
                        }
                        _ => {
                            TerminalEditorResult::Continue
                        }
                    }
                }
                ListCursorMode::Modify => {
                    let mut ce = self.data.get_mut(idx);
                    let mut cur_edit = ce.write().unwrap();
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Char(' '))) => {
                            // split..
                            cur_edit.up();
                            drop(cur_edit);
                            self.cursor.set(ListCursor {
                                mode: ListCursorMode::Insert,
                                idx: Some(idx+1)
                            });
                            TerminalEditorResult::Continue
                        }
                        TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                            match cur_edit.handle_terminal_event(event) {
                                TerminalEditorResult::Exit => {
                                    drop(cur_edit);
                                    drop(ce);
                                    self.cursor.set(ListCursor {
                                        mode: ListCursorMode::Insert,
                                        idx: Some(idx)
                                    });
                                    self.data.remove(idx); // todo: join instead of remove
                                }
                                TerminalEditorResult::Continue => {
                                }
                            }
                            TerminalEditorResult::Continue
                        }
                        _ => cur_edit.handle_terminal_event(event)
                    }
                }
            }
        } else {
            TerminalEditorResult::Continue
        }
    }
}

enum ListEditorViewSegment {
    InsertCursor,
    View(OuterViewPort<dyn TerminalView>),
    Select(OuterViewPort<dyn TerminalView>),
    Modify(OuterViewPort<dyn TerminalView>)
}

struct ListEditorView {
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
            ListCursorMode::Insert => Some(self.data.len()? + 1),
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
    fn new(
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

pub enum ListEditorStyle {
    HorizontalSexpr,
    VerticalSexpr,
    Path,
    String,
    Clist,
    Hex,
    Plain
}

impl<ItemEditor, FnMakeItemEditor> ListEditor<ItemEditor, FnMakeItemEditor>
where ItemEditor: TerminalEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<ItemEditor>>
{
    pub fn get_seg_seq_view(&self) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        self.segment_seq
            .map(
                |segment| match segment {
                    ListEditorViewSegment::InsertCursor =>
                        make_label("|")
                        .map_item(
                            |_pt, atom|
                            atom.add_style_back(TerminalStyle::fg_color((90,60,200)))
                                .add_style_back(TerminalStyle::bg_color((0,0,0)))
                                .add_style_back(TerminalStyle::bold(true))
                        ),
                    ListEditorViewSegment::Select(sub_view) =>
                        sub_view.map_item(
                            |_pt, atom|
                            atom.add_style_front(TerminalStyle::bg_color((90,60,200)))
                        ),
                    ListEditorViewSegment::Modify(sub_view) =>
                        sub_view.map_item(
                            |_pt, atom|
                            atom.add_style_back(TerminalStyle::bg_color((0,0,0)))
                                .add_style_back(TerminalStyle::bold(true))
                        ),
                    ListEditorViewSegment::View(sub_view) =>
                        sub_view.clone()
                }
            )
    }

    pub fn horizontal_sexpr_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view().horizontal_sexpr_view(0)
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

    pub fn hex_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .decorate("0x", "", "", 0)
            .to_grid_horizontal()
            .flatten()
    }

    pub fn plain_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_seg_seq_view()
            .to_grid_horizontal()
            .flatten()
    }
    
    pub fn new(make_item_editor: FnMakeItemEditor, style: ListEditorStyle) -> Self {
        let cursor_port = ViewPort::new();
        let data_port = ViewPort::new();

        let mut cursor = SingletonBuffer::new(ListCursor::default(), cursor_port.inner());
        let mut data = VecBuffer::<Arc<RwLock<ItemEditor>>>::new(data_port.inner());

        let data_sequence_port = data_port.into_outer().to_sequence();

        let segment_view_port = ViewPort::<dyn SequenceView<Item = ListEditorViewSegment>>::new();
        let segment_view = ListEditorView::new(
            cursor_port.outer(),
            data_sequence_port.map(|ed| ed.read().unwrap().get_term_view()),
            segment_view_port.inner()
        );

        let mut le = ListEditor {
            data,
            data_sequence_port,
            cursor,
            make_item_editor,
            level: 0,
            segment_seq: segment_view_port.outer(),
            terminal_view: make_label("lol"),
        };
        le.set_style(style);
        le
    }

    pub fn set_style(&mut self, style: ListEditorStyle) {
        self.terminal_view = match style {
            ListEditorStyle::HorizontalSexpr => self.horizontal_sexpr_view(),
            ListEditorStyle::VerticalSexpr => self.vertical_sexpr_view(),
            ListEditorStyle::Path => self.path_view(),
            ListEditorStyle::String => self.string_view(),
            ListEditorStyle::Clist => self.clist_view(),
            ListEditorStyle::Hex => self.hex_view(),
            ListEditorStyle::Plain => self.plain_view()
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = Arc<RwLock<ItemEditor>>>> {
        self.data_sequence_port.clone()
    }

    fn get_item(&self) -> Option<Arc<RwLock<ItemEditor>>> {
        if let Some(idx) = self.cursor.get().idx {
            Some(self.data.get(idx))
        } else {
            None
        }
    }

    fn set_idx(&mut self, idx: isize) {
        let cur = self.cursor.get();
        let mode = cur.mode;

        if idx < 0 {
            self.cursor.set(ListCursor {
                mode,
                idx: Some((self.data.len() as isize + idx) as usize)
            });
        } else {
            self.cursor.set(ListCursor {
                mode,
                idx: Some(idx as usize)
            });
        }
    }

    fn set_mode(&mut self, mode: ListCursorMode) {
        let mut cur = self.cursor.get();

        if cur.mode == ListCursorMode::Insert &&
            mode != ListCursorMode::Insert
        {
            if let Some(idx) = cur.idx {
                if idx == self.data.len() && idx > 0 {
                    cur.idx = Some(idx-1);
                }
            }
        }

        cur.mode = mode;
        self.cursor.set(cur);
    }
}

