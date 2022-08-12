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
        index::{buffer::{IndexBuffer, MutableIndexAccess}, IndexView},
        list::ListCursorMode,
        product::{segment::ProductEditorSegment},
        make_editor::make_editor
    },
    cgmath::{Vector2, Point2},
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    std::ops::{Deref, DerefMut}
};

pub struct ProductEditor {
    segments: IndexBuffer<Point2<i16>, ProductEditorSegment>,
    pub(super) n_indices: Vec<Point2<i16>>,

    pub(super) ctx: Arc<RwLock<Context>>,    
    pub(super) cursor: Option<isize>,
    pub(super) depth: usize,
}

impl ProductEditor {
    pub fn new(depth: usize, ctx: Arc<RwLock<Context>>) -> Self {
        ProductEditor {
            segments: IndexBuffer::new(),
            n_indices: Vec::new(),
            ctx,
            cursor: None,
            depth
        }
    }
    
    pub fn with_t(mut self, pos: Point2<i16>, t: &str) -> Self {
        self.segments.insert(pos, ProductEditorSegment::T(t.to_string(), self.depth));
        self
    }

    pub fn with_n(mut self, pos: Point2<i16>, n: TypeLadder) -> Self {
        self.segments.insert(pos, ProductEditorSegment::N{
            t: n,
            editor: None,
            cur_depth: 0
        });
        self.n_indices.push(pos);
        self
    }

    pub fn get_editor_segment(&self, mut idx: isize) -> ProductEditorSegment {
        idx = crate::modulo(idx, self.n_indices.len() as isize);
        if let Some(pos) = self.n_indices.get(idx as usize) {
            self.segments.get(pos).unwrap()
        } else {
            unreachable!()
        }
    }

    pub fn get_editor_segment_mut(&mut self, mut idx: isize) -> MutableIndexAccess<Point2<i16>, ProductEditorSegment> {
        idx = crate::modulo(idx, self.n_indices.len() as isize);
        if let Some(pos) = self.n_indices.get(idx as usize) {
            self.segments.get_mut(pos)
        } else {
            unreachable!()
        }
    }

    pub fn get_cur_segment(&self) -> Option<ProductEditorSegment> {
        Some(self.get_editor_segment(self.cursor?))
    }

    pub fn get_cur_segment_mut(&mut self) -> Option<MutableIndexAccess<Point2<i16>, ProductEditorSegment>> {
        Some(self.get_editor_segment_mut(self.cursor?))
    }

    pub fn get_editor(&self, idx: isize) -> Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>> {
        if let ProductEditorSegment::N{ t: _, editor, cur_depth: _ } = self.get_editor_segment(idx) {
            editor
        } else {
            unreachable!()
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
            .map_item(move |_pos, e: &ProductEditorSegment| { e.get_view(ctx.clone()) })
            .flatten()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        if let Some(mut segment) = self.get_cur_segment_mut().as_deref_mut() {
            if let Some(ProductEditorSegment::N{ t, editor, cur_depth }) = segment.deref_mut() {
            *cur_depth = self.get_cursor().tree_addr.len();
            if let Some(e) = editor.clone() {
                let mut ce = e.write().unwrap();
                match ce.handle_terminal_event(event) {
                    TerminalEditorResult::Exit =>
                        match event {
                            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                *editor = None;
                                *cur_depth = 1;
                                TerminalEditorResult::Continue
                            }
                            _ => {
                                *cur_depth = ce.get_cursor().tree_addr.len();
                                drop(ce);
                                match self.nexd() {
                                    TreeNavResult::Continue => TerminalEditorResult::Continue,
                                    TreeNavResult::Exit => TerminalEditorResult::Exit
                                }
                            }
                        },
                    TerminalEditorResult::Continue => {
                        *cur_depth = ce.get_cursor().tree_addr.len();
                        TerminalEditorResult::Continue
                    }
                }
            } else {
                let e = make_editor(self.ctx.clone(), t, self.depth+1);
                *editor = Some(e.clone());
                e.write().unwrap().dn();
                let x = e.write().unwrap().handle_terminal_event(event);
                *cur_depth = e.write().unwrap().get_cursor().tree_addr.len();
                x
            }
            } else {
                unreachable!()
            }
        } else {
            TerminalEditorResult::Exit
        }
    }
}

