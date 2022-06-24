use {
    crate::{
        core::{ViewPort, OuterViewPort, TypeLadder, Context},
        terminal::{
            TerminalEditor, TerminalEditorResult,
            TerminalEvent, TerminalView
        },
        sequence::{SequenceView},
        tree_nav::{TreeNav, TerminalTreeEditor, TreeNavResult},
        vec::{VecBuffer, MutableVecAccess},
        list::ListCursorMode,
        product::{segment::ProductEditorSegment},
        make_editor::make_editor
    },
    cgmath::Vector2,
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

pub struct ProductEditor {
    segments: VecBuffer<ProductEditorSegment>,
    pub(super) n_indices: Vec<usize>,
    
    pub(super) ctx: Arc<RwLock<Context>>,    
    pub(super) cursor: Option<isize>,
    pub(super) depth: usize,
}

impl ProductEditor {
    pub fn new(depth: usize, ctx: Arc<RwLock<Context>>) -> Self {
        ProductEditor {
            segments: VecBuffer::new(),
            n_indices: Vec::new(),
            ctx,
            cursor: None,
            depth
        }
    }
    
    pub fn with_t(mut self, t: &str) -> Self {
        self.segments.push(ProductEditorSegment::T(t.to_string(), self.depth));
        self
    }

    pub fn with_n(mut self, n: TypeLadder) -> Self {
        let elem_idx = self.segments.len();
        self.segments.push(ProductEditorSegment::N{
            t: n,
            editor: None,
            cur_depth: 0
        });
        self.n_indices.push(elem_idx);
        self
    }

    pub fn get_editor_element(&self, mut idx: isize) -> Option<ProductEditorSegment> {
        idx = crate::modulo(idx, self.n_indices.len() as isize);
        if let Some(i) = self.n_indices.get(idx as usize) {
            Some(self.segments.get(*i))
        } else {
            None
        }
    }

    pub fn get_editor_element_mut(&mut self, mut idx: isize) -> Option<MutableVecAccess<ProductEditorSegment>> {
        idx = crate::modulo(idx, self.n_indices.len() as isize);
        if let Some(i) = self.n_indices.get(idx as usize) {
            Some(self.segments.get_mut(*i))
        } else {
            None
        }
    }

    pub fn get_cur_element(&self) -> Option<ProductEditorSegment> {
        self.get_editor_element(self.cursor?)
    }

    pub fn get_cur_element_mut(&mut self) -> Option<MutableVecAccess<ProductEditorSegment>> {
        self.get_editor_element_mut(self.cursor?)
    }

    pub fn get_editor(&self, idx: isize) -> Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>> {
        if let Some(ProductEditorSegment::N{ t: _, editor, cur_depth: _ }) = self.get_editor_element(idx) {
            editor
        } else {
            None
        }
    }

    pub fn get_cur_editor(&self) -> Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>> {
        self.get_editor(self.cursor?)
    }

    pub fn set_leaf_mode(&mut self, mode: ListCursorMode) {
        let mut c = self.get_cursor();
        c.leaf_mode = mode;
        self.goto(c);
    }
}

impl TerminalEditor for ProductEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        let ctx = self.ctx.clone();
        self.segments
            .get_port()
            .to_sequence()
            .map(move |e: &ProductEditorSegment| { e.get_view(ctx.clone()) })
            .to_grid_horizontal()
            .flatten()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = self.get_cur_element_mut().as_deref_mut() {
            *cur_depth = self.get_cursor().tree_addr.len();
            if let Some(e) = editor.clone() {
                let mut ce = e.write().unwrap();
                match ce.handle_terminal_event(event) {
                    TerminalEditorResult::Exit =>
                        match event {
                            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                *editor = None;
                                *cur_depth -= 1;
                                TerminalEditorResult::Continue
                            }
                            _ => {
                                drop(ce);
                                match self.nexd() {
                                    TreeNavResult::Continue => TerminalEditorResult::Continue,
                                    TreeNavResult::Exit => TerminalEditorResult::Exit
                                }
                            }
                        },
                    TerminalEditorResult::Continue =>
                    TerminalEditorResult::Continue
                }
            } else {
                let e = make_editor(self.ctx.clone(), t, self.depth+1);
                *editor = Some(e.clone());
                e.write().unwrap().dn();
                let x = e.write().unwrap().handle_terminal_event(event);
                *cur_depth = self.get_cursor().tree_addr.len()+1;
                x
            }
        } else {
            TerminalEditorResult::Exit
        }
    }
}

