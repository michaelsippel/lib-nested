use {
    crate::{
        core::{ViewPort, OuterViewPort, TypeLadder, Context},
        terminal::{
            TerminalEditor, TerminalEditorResult,
            TerminalEvent, TerminalView
        },
        vec::{VecBuffer, MutableVecAccess},
        index::{buffer::{IndexBuffer, MutableIndexAccess}, IndexView},
        list::ListCursorMode,
        product::{segment::ProductEditorSegment},
        sequence::{SequenceView},
        tree::{TreeNav, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        terminal::{TerminalStyle},
        Nested
    },
    cgmath::{Vector2, Point2},
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    std::ops::{Deref, DerefMut}
};

pub struct ProductEditor {
    msg_buf: VecBuffer<Option<OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>>>>,
    msg_port:  OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>>,
    segments: IndexBuffer<Point2<i16>, ProductEditorSegment>,
    pub(super) n_indices: Vec<Point2<i16>>,

    pub(super) ctx: Arc<RwLock<Context>>,    
    pub(super) cursor: Option<isize>,
    pub(super) depth: usize,
}

impl ProductEditor {
    pub fn new(depth: usize, ctx: Arc<RwLock<Context>>) -> Self {
        let msg_buf = VecBuffer::new();
        ProductEditor {
            segments: IndexBuffer::new(),
            msg_port: msg_buf.get_port()
                .to_sequence()
                .enumerate()
                .filter_map(|(idx, msgs): &(usize, Option<OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>>>)| {
                    let idx = *idx;
                    if let Some(msgs) = msgs {
                        Some(msgs.map(
                            move |msg| {
                                let mut msg = msg.clone();
                                msg.addr.insert(0, idx);
                                msg
                            }))
                    } else {
                        None
                    }
                })
                .flatten(),
            msg_buf,

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
            t: n.clone(),
            editor: None,
            ed_depth: self.depth + 1,
            cur_depth: 0,
            cur_dist: isize::MAX
        });
        self.n_indices.push(pos);

        let mut b = VecBuffer::new();
        b.push(crate::diagnostics::make_todo(crate::terminal::make_label(&format!("complete {}", self.ctx.read().unwrap().type_term_to_str(&n[0])))));
        self.msg_buf.push(Some(b.get_port().to_sequence()));
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

    pub fn get_editor(&self, idx: isize) -> Option<Arc<RwLock<dyn Nested + Send + Sync>>> {
        if let ProductEditorSegment::N{ t: _, editor, ed_depth: _, cur_depth: _, cur_dist: _ } = self.get_editor_segment(idx) {
            editor
        } else {
            unreachable!()
        }
    }

    pub fn get_cur_editor(&self) -> Option<Arc<RwLock<dyn Nested + Send + Sync>>> {
        self.get_editor(self.cursor?)
    }

    pub fn set_leaf_mode(&mut self, mode: ListCursorMode) {
        let mut c = self.get_cursor();
        c.leaf_mode = mode;
        self.goto(c);
    }

    pub fn update_segment(&mut self, idx: isize) {
        if let Some(ProductEditorSegment::N{ t, editor, ed_depth: _, cur_depth, cur_dist }) = self.get_editor_segment_mut(idx).deref_mut() {
            let cur = self.get_cursor();

            if cur.tree_addr.len() > 0 {
                if cur.tree_addr[0] == idx {
                    *cur_depth = cur.tree_addr.len();
                }
                
                *cur_dist = cur.tree_addr[0] - idx
            } else {
                *cur_dist = isize::MAX;
            };

            if let Some(e) = editor {
                self.msg_buf.update(idx as usize, Some(e.read().unwrap().get_msg_port()));
            } else {
                let mut b = VecBuffer::new();
                b.push(crate::diagnostics::make_todo(crate::terminal::make_label(&format!("complete {}", self.ctx.read().unwrap().type_term_to_str(&t[0])))));

                self.msg_buf.update(idx as usize, Some(b.get_port().to_sequence()));

                if cur.tree_addr.len() > 0 {
                    if cur.tree_addr[0] == idx {
                        self.msg_buf.update(idx as usize, Some(b.get_port().to_sequence().map(
                            |msg| {
                                let mut msg = msg.clone();
                                msg.port = msg.port.map_item(|p,a| a.add_style_back(TerminalStyle::bg_color((40,40,40))));
                                msg
                            }
                        )));

                    }
                }
            }

        } else {
            unreachable!()
        }
    }

    pub fn update_cur_segment(&mut self) {
        if let Some(c) = self.cursor {
            self.update_segment(c);
        }
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
        let mut update_segment = false;

        let result = if let Some(mut segment) = self.get_cur_segment_mut().as_deref_mut() {
            if let Some(ProductEditorSegment::N{ t, editor, ed_depth, cur_depth, cur_dist }) = segment.deref_mut() {
                *cur_depth = self.get_cursor().tree_addr.len();

                if let Some(e) = editor.clone() {
                    let mut ce = e.write().unwrap();
                    match ce.handle_terminal_event(event) {
                        TerminalEditorResult::Exit =>
                            match event {
                                TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                    *editor = None;
                                    update_segment = true;
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
                    let e = Context::make_editor(self.ctx.clone(), t[0].clone(), *ed_depth+1).unwrap();
                    *editor = Some(e.clone());
                    update_segment = true;

                    e.write().unwrap().dn();
                    let x = e.write().unwrap().handle_terminal_event(event);
                    x
                }
            } else {
                unreachable!();
                TerminalEditorResult::Exit
            }
        } else {
            TerminalEditorResult::Exit
        };

        if update_segment {
            self.update_cur_segment();
        }
        result
    }
}

impl Diagnostics for ProductEditor {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = crate::diagnostics::Message>> {
        self.msg_port.clone()
    }
}


