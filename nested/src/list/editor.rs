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
        tree_nav::{TreeNav, TreeNavResult, TerminalTreeEditor},
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
        string_editor::StringEditor,
        leveled_term_view::LeveledTermView,
        list::{SExprView, ListDecoration}
    }
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ListEditorCursor {
    None,
    Insert(usize),
    Select(usize),
    Edit(usize)
}

impl Default for ListEditorCursor {
    fn default() -> Self {
        ListEditorCursor::None
    }
}

pub struct ListEditor<SubEditor, FnMakeItemEditor>
where SubEditor: TerminalEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<SubEditor>>
{
    cursor: SingletonBuffer<ListEditorCursor>,
    data: VecBuffer<Arc<RwLock<SubEditor>>>,
    data_sequence_port: OuterViewPort<dyn SequenceView<Item = Arc<RwLock<SubEditor>>>>,
    make_item_editor: FnMakeItemEditor,
    level: usize,
    segment_seq: OuterViewPort<dyn SequenceView<Item = ListEditorViewSegment>>,

    terminal_view: OuterViewPort<dyn TerminalView>
}

impl<SubEditor, FnMakeItemEditor> TreeNav for ListEditor<SubEditor, FnMakeItemEditor>
where SubEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<SubEditor>>
{
    fn get_cursor(&self) -> Option<Vec<usize>> {
        match self.cursor.get() {
            ListEditorCursor::None => None,
            ListEditorCursor::Insert(idx) => Some(vec![ idx ]),
            ListEditorCursor::Select(idx) => Some(vec![ idx ]),
            ListEditorCursor::Edit(idx) => match self.data.get(idx).read().unwrap().get_cursor() {
                Some(mut addr) => {
                    addr.insert(0, idx);
                    Some(addr)
                }
                None => Some(vec![ idx ])
            },
        }
    }

    fn up(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::Edit(idx) => {
                let ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                match cur_edit.up() {
                    TreeNavResult::Exit => {
                        self.cursor.set(ListEditorCursor::Select(idx));
                    }
                    TreeNavResult::Continue => {}
                }
                TreeNavResult::Continue
            }
            _ => {
                self.cursor.set(ListEditorCursor::None);
                TreeNavResult::Exit
            }
        }
    }

    fn goto(&mut self, tree_addr: Vec<usize>) -> TreeNavResult {
        if tree_addr.len() == 1 {
            if tree_addr[0] < self.data.len() {
                match self.cursor.get() {
                    ListEditorCursor::None |
                    ListEditorCursor::Select(_) => {
                        self.cursor.set(ListEditorCursor::Select(tree_addr[0]));
                    }
                    ListEditorCursor::Insert(_) => {
                        self.cursor.set(ListEditorCursor::Insert(tree_addr[0]));
                    }
                    ListEditorCursor::Edit(_) => {
                        self.cursor.set(ListEditorCursor::Edit(tree_addr[0]));
                    }
                }
                TreeNavResult::Continue
            } else {
                TreeNavResult::Exit
            }                    
        } else if tree_addr.len() > 0 {
            if tree_addr[0] < self.data.len() {
                self.cursor.set(ListEditorCursor::Edit(tree_addr[0]));

                let ce = self.data.get_mut(tree_addr[0]);
                let mut cur_edit = ce.write().unwrap();

                cur_edit.goto(tree_addr[1..].iter().cloned().collect());

                TreeNavResult::Continue
            } else {
                TreeNavResult::Exit
            }
        } else {
            self.cursor.set(ListEditorCursor::None);
            TreeNavResult::Exit
        }
    }

    fn goto_end(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::None |
            ListEditorCursor::Insert(_) | 
            ListEditorCursor::Select(_) =>
                self.goto(vec![ self.data.len()-1 ]),
            ListEditorCursor::Edit(idx) => {
                let ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                match cur_edit.goto_end() {
                    TreeNavResult::Continue => TreeNavResult::Continue,
                    TreeNavResult::Exit => {
                        if idx+1 < self.data.len() {
                            self.cursor.set(ListEditorCursor::Edit(idx+1));
                            self.data.get_mut(idx+1).write().unwrap().goto_end();
                            TreeNavResult::Continue
                        } else {
                            self.cursor.set(ListEditorCursor::None);
                            TreeNavResult::Exit
                        }
                    }
                }                
            }
        }
    }

    fn goto_home(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::None |
            ListEditorCursor::Insert(_) | 
            ListEditorCursor::Select(_) => self.goto(vec![ 0 ]),
            ListEditorCursor::Edit(idx) => {
                let ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                match cur_edit.goto_home() {
                    TreeNavResult::Continue => TreeNavResult::Continue,
                    TreeNavResult::Exit => {
                        if idx > 0 {
                            self.cursor.set(ListEditorCursor::Edit(idx-1));
                            self.data.get_mut(idx-1).write().unwrap().goto_home();
                            TreeNavResult::Continue
                        } else {
                            self.cursor.set(ListEditorCursor::None);
                            TreeNavResult::Exit
                        }
                    }
                }
            }
            _ => {
                self.up();
                TreeNavResult::Exit
            }
        }
    }

    fn dn(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::Select(idx) => {
                self.data.get_mut(idx).write().unwrap().goto_home();
                self.cursor.set(ListEditorCursor::Edit(idx));
            }
            _ => {}
        }
        TreeNavResult::Continue
    }

    fn pxev(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::None => TreeNavResult::Exit,
            ListEditorCursor::Insert(idx) => {
                if idx > 0 {
                    self.cursor.set(ListEditorCursor::Insert(idx-1));
                    TreeNavResult::Continue
                } else {
                    self.cursor.set(ListEditorCursor::None);
                    TreeNavResult::Exit
                }
            }
            ListEditorCursor::Select(idx) => {
                if idx > 0 {
                    self.cursor.set(ListEditorCursor::Select(idx-1));
                    TreeNavResult::Continue
                } else {
                    self.cursor.set(ListEditorCursor::None);
                    TreeNavResult::Exit
                }
            }
            ListEditorCursor::Edit(idx) => {
                let ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                match cur_edit.pxev() {
                    TreeNavResult::Exit => {
                        if idx > 0 {
                            self.cursor.set(ListEditorCursor::Edit(idx-1));
                            self.data.get_mut(idx-1).write().unwrap().goto_end();
                            TreeNavResult::Continue
                        } else {
                            TreeNavResult::Exit
                        }
                    }
                    TreeNavResult::Continue => TreeNavResult::Continue
                }
            }
        }
    }

    fn nexd(&mut self) -> TreeNavResult {
        match self.cursor.get() {
            ListEditorCursor::None => TreeNavResult::Exit,
            ListEditorCursor::Insert(idx) => {
                if idx < self.data.len() {
                    self.cursor.set(ListEditorCursor::Insert(idx+1));
                    TreeNavResult::Continue
                } else {
                    TreeNavResult::Exit
                }
            }
            ListEditorCursor::Select(idx) => {
                if idx+1 < self.data.len() {
                    self.cursor.set(ListEditorCursor::Select(idx + 1));
                    TreeNavResult::Continue
                } else {
                    TreeNavResult::Exit
                }
            }
            ListEditorCursor::Edit(idx) => {
                let ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                match cur_edit.nexd() {
                    TreeNavResult::Exit => {
                        if idx+1 < self.data.len() {
                            self.cursor.set(ListEditorCursor::Edit(idx+1));
                            self.data.get_mut(idx+1).write().unwrap().goto_home();
                            TreeNavResult::Continue
                        } else {//if idx+1 == self.data.len() {
                            TreeNavResult::Exit
                        }
                    }
                    TreeNavResult::Continue => TreeNavResult::Continue
                }
            }
        }
    }
}

impl<SubEditor, FnMakeItemEditor> TerminalEditor for ListEditor<SubEditor, FnMakeItemEditor>
where SubEditor: TerminalTreeEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<SubEditor>>
{
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.horizontal_sexpr_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match self.cursor.get() {
            ListEditorCursor::Insert(idx) => {
                self.data.insert(idx, (self.make_item_editor)());

                let mut ce = self.data.get_mut(idx);
                let mut cur_edit = ce.write().unwrap();

                cur_edit.goto_home();
                self.cursor.set(ListEditorCursor::Edit(idx));
                cur_edit.handle_terminal_event(event);
            }
            ListEditorCursor::Edit(idx) => {
                match event {
                    TerminalEvent::Input(Event::Key(Key::Char(' '))) => {
                        // split..
                        self.data.insert(idx+1, (self.make_item_editor)());
                        self.data.get_mut(idx).write().unwrap().goto_end();
                        self.data.get_mut(idx+1).write().unwrap().goto_home();
                        self.cursor.set(ListEditorCursor::Edit(idx+1));
                    }
                    event => {
                        let mut ce = self.data.get_mut(idx);
                        let mut cur_edit = ce.write().unwrap();

                        cur_edit.handle_terminal_event(event);
                    }
                }
            }
            ListEditorCursor::Select(idx) => {
                match event {
                    TerminalEvent::Input(Event::Key(Key::Insert)) => {
                        
                    }
                    TerminalEvent::Input(Event::Key(Key::Delete)) => {
                        self.data.remove(idx);

                        if self.data.len() == 0 {
                            self.cursor.set(ListEditorCursor::Insert(0));
                        } else if idx == self.data.len() {
                            self.cursor.set(ListEditorCursor::Select(idx-1));
                        }
                    }
                    _=>{}
                }
            }
            _ => {}
        }

        TerminalEditorResult::Continue
    }
}

enum ListEditorViewSegment {
    InsertCursor,
    View(OuterViewPort<dyn TerminalView>),
    Select(OuterViewPort<dyn TerminalView>),
    Edit(OuterViewPort<dyn TerminalView>)
}

struct ListEditorView {
    cursor: Arc<dyn SingletonView<Item = ListEditorCursor>>,
    data: Arc<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
    cur_cursor: ListEditorCursor,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = ListEditorViewSegment>>>>,
    proj_helper: ProjectionHelper<Self>
}

impl View for ListEditorView {
    type Msg = usize;
}

impl SequenceView for ListEditorView {
    type Item = ListEditorViewSegment;

    fn len(&self) -> Option<usize> {
        match self.cur_cursor {
            ListEditorCursor::None => self.data.len(),
            ListEditorCursor::Select(_) => self.data.len(),
            ListEditorCursor::Edit(_) => self.data.len(),
            ListEditorCursor::Insert(_) => Some(self.data.len()? + 1)
        }
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        Some(
            match self.cur_cursor {
                ListEditorCursor::None =>
                    ListEditorViewSegment::View(self.data.get(idx)?),
                ListEditorCursor::Select(c) =>
                    if *idx == c {
                        ListEditorViewSegment::Select(self.data.get(idx)?)
                    } else {
                        ListEditorViewSegment::View(self.data.get(idx)?)
                    },
                ListEditorCursor::Edit(c) =>
                    if *idx == c {
                        ListEditorViewSegment::Edit(self.data.get(idx)?)
                    } else {
                        ListEditorViewSegment::View(self.data.get(idx)?)
                    },
                ListEditorCursor::Insert(c) =>
                    if *idx < c {
                        ListEditorViewSegment::View(self.data.get(idx)?)
                    } else if *idx == c {
                        ListEditorViewSegment::InsertCursor
                    } else {
                        ListEditorViewSegment::View(self.data.get(&(idx-1))?)
                    },
            }
        )
    }
}

impl ListEditorView {
    fn new(
        cursor_port: OuterViewPort<dyn SingletonView<Item = ListEditorCursor>>,
        data_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = ListEditorViewSegment>>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(
                ListEditorView {
                    cur_cursor: ListEditorCursor::None,
                    cursor: proj_helper.new_singleton_arg(
                        cursor_port,
                        |s: &mut Self, _msg| {
                            let old_cursor = s.cur_cursor;
                            let new_cursor = s.cursor.get();
                            s.cur_cursor = new_cursor;

                            let mut begin = std::cmp::min(
                                match old_cursor {
                                    ListEditorCursor::None => usize::MAX,
                                    ListEditorCursor::Select(c) => c,
                                    ListEditorCursor::Insert(c) => c,
                                    ListEditorCursor::Edit(c) => c
                                },
                                match new_cursor {
                                    ListEditorCursor::None => usize::MAX,
                                    ListEditorCursor::Select(c) => c,
                                    ListEditorCursor::Insert(c) => c,
                                    ListEditorCursor::Edit(c) => c
                                }
                            );
                            let mut end =
                                /*
                                match (old_cursor, new_cursor) {
                                    (ListEditorCursor::None, ListEditorCursor::None) => usize::MAX,
                                    (ListEditorCursor::Select(old_pos), ListEditorCursor::Select(new_pos)) => max(old_pos, new_pos),
                                    (ListEditorCursor::Edit(old_pos), ListEditorCursor::Edit(new_pos)) => max(old_pos, new_pos),
                                    (ListEditorCursor::Insert(old_pos), ListEditorCursor::Insert(new_pos)) => max(old_pos, new_pos),
                                    (ListEditorCursor::)
                                };
*/
                                std::cmp::max(
                                match old_cursor {
                                    ListEditorCursor::None => 0,
                                    ListEditorCursor::Select(c) => c,
                                    ListEditorCursor::Insert(c) => c+1,
                                    ListEditorCursor::Edit(c) => c
                                },
                                match new_cursor {
                                    ListEditorCursor::None => 0,
                                    ListEditorCursor::Select(c) => c,
                                    ListEditorCursor::Insert(c) => c+1,
                                    ListEditorCursor::Edit(c) => c
                                }
                            );

                            s.cast.notify_each(
                                begin ..= end
                            );
                        }),
                    data: proj_helper.new_sequence_arg(
                        data_port,
                        |s: &mut Self, idx| {
                            match s.cur_cursor {
                                ListEditorCursor::None => s.cast.notify(idx),
                                ListEditorCursor::Select(c) => s.cast.notify(idx),
                                ListEditorCursor::Edit(c) => s.cast.notify(idx),
                                ListEditorCursor::Insert(c) =>
                                    if *idx < c {
                                        s.cast.notify(idx)
                                    } else {
                                        s.cast.notify(&(*idx + 1))
                                    }
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

impl<SubEditor, FnMakeItemEditor> ListEditor<SubEditor, FnMakeItemEditor>
where SubEditor: TerminalEditor + ?Sized + Send + Sync + 'static,
      FnMakeItemEditor: Fn() -> Arc<RwLock<SubEditor>>
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
                                .add_style_back(TerminalStyle::bold(true))
                        ),
                    ListEditorViewSegment::Select(sub_view) =>
                        sub_view.map_item(
                            |_pt, atom|
                            atom.add_style_back(TerminalStyle::bg_color((90,60,200)))
                        ),
                    ListEditorViewSegment::Edit(sub_view) =>
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

    pub fn new(make_item_editor: FnMakeItemEditor) -> Self {
        let cursor_port = ViewPort::new();
        let data_port = ViewPort::new();

        let mut cursor = SingletonBuffer::new(ListEditorCursor::Insert(0), cursor_port.inner());
        let mut data = VecBuffer::<Arc<RwLock<SubEditor>>>::new(data_port.inner());

        let data_sequence_port = data_port.into_outer().to_sequence();

        let segment_view_port = ViewPort::<dyn SequenceView<Item = ListEditorViewSegment>>::new();
        let segment_view = ListEditorView::new(
            cursor_port.outer(),
            data_sequence_port.map(|ed| ed.read().unwrap().get_term_view()),
            segment_view_port.inner()
        );

        ListEditor {
            data,
            data_sequence_port,
            cursor,
            make_item_editor,
            level: 0,
            segment_seq: segment_view_port.outer()
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = Arc<RwLock<SubEditor>>>> {
        self.data_sequence_port.clone()
    }
}

