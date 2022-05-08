
use {
    crate::{
        core::{ViewPort, OuterViewPort, Observer, port::UpdateTask, TypeTerm, TypeLadder, Context},
        terminal::{
            Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView,
            make_label
        },
        sequence::{SequenceView},
        tree_nav::{TreeNav, TerminalTreeEditor, TreeCursor, TreeNavResult},
        vec::{VecBuffer, MutableVecAccess},
        index::buffer::IndexBuffer,
        integer::PosIntEditor,
        string_editor::{StringEditor, CharEditor},
        list::{ListEditor, ListCursorMode, ListEditorStyle},
        product::{element::ProductEditorElement},
        make_editor::make_editor
    },
    cgmath::{Point2, Vector2},
    std::{sync::{Arc, RwLock}, ops::{Deref, DerefMut}},
    termion::event::{Event, Key},
};

pub struct ProductEditor {
    elements: VecBuffer<ProductEditorElement>,
    n_indices: Vec<usize>,
    
    el_port: OuterViewPort<dyn SequenceView<Item = ProductEditorElement>>,
    el_view_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,

    ctx: Arc<RwLock<Context>>,
    
    cursor: Option<usize>,
    depth: usize
}

impl ProductEditor {
    pub fn new(depth: usize, ctx: Arc<RwLock<Context>>) -> Self {
        let mut port = ViewPort::new();

        let el_port = port.outer().to_sequence();
        let el_view_port = el_port.map({
            let ctx = ctx.clone();
            move |e: &ProductEditorElement| { e.get_view(ctx.clone()) }
        });

        ProductEditor {
            elements: VecBuffer::new(port.inner()),
            el_port,
            el_view_port,
            n_indices: Vec::new(),

            ctx,

            cursor: None,
            depth
        }
    }

    pub fn with_t(mut self, t: &str) -> Self {
        self.elements.push(ProductEditorElement::T(t.to_string()));
        self
    }

    pub fn with_n(mut self, n: TypeLadder) -> Self {
        let elem_idx = self.elements.len();
        self.elements.push(ProductEditorElement::N{
            t: n,
            editor: None,
            select: false
        });
        self.n_indices.push(elem_idx);
        self
    }

    fn get_editor_element(&self, idx: usize) -> Option<ProductEditorElement> {
        if let Some(i) = self.n_indices.get(idx) {
            Some(self.elements.get(*i))
        } else {
            None
        }
    }

    fn get_editor_element_mut(&mut self, idx: usize) -> Option<MutableVecAccess<ProductEditorElement>> {
        if let Some(i) = self.n_indices.get(idx) {
            Some(self.elements.get_mut(*i))
        } else {
            None
        }
    }

    fn get_cur_element(&self) -> Option<ProductEditorElement> {
        self.get_editor_element(self.cursor?)
    }

    fn get_cur_element_mut(&mut self) -> Option<MutableVecAccess<ProductEditorElement>> {
        self.get_editor_element_mut(self.cursor?)
    }

    fn get_editor(&self, idx: usize) -> Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>> {
        if let Some(ProductEditorElement::N{ t: _, editor, select: _ }) = self.get_editor_element(idx) {
            editor
        } else {
            None
        }
    }
    
    fn get_cur_editor(&self) -> Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>> {
        self.get_editor(self.cursor?)
    }

    fn set_leaf_mode(&mut self, mode: ListCursorMode) {
        let mut c = self.get_cursor();
        c.leaf_mode = mode;
        self.goto(c);
    }
}

impl TreeNav for ProductEditor {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(i) = self.cursor {
            if let Some(e) = self.get_editor(i) {
                let mut c = e.read().unwrap().get_cursor();
                if c.tree_addr.len() == 0 {
                    c.leaf_mode = ListCursorMode::Select;
                }
                c.tree_addr.insert(0, i);                
                c
            } else {
                TreeCursor {
                    leaf_mode: ListCursorMode::Select,
                    tree_addr: vec![ i ]
                }
            }
        } else {
            TreeCursor::default()
        }
    }

    fn goto(&mut self, mut c: TreeCursor) -> TreeNavResult {
        if let Some(mut element) = self.get_cur_element_mut() {
            if let ProductEditorElement::N{ t, editor, select } = element.deref_mut() {
                if let Some(e) = editor {
                    e.write().unwrap().goto(TreeCursor::default());
                }
                *select = false;
            }
        }

        if c.tree_addr.len() > 0 {
            self.cursor = Some(c.tree_addr.remove(0));

            if let Some(mut element) = self.get_cur_element_mut() {
                if let ProductEditorElement::N{ t, editor, select } = element.deref_mut() {
                    if let Some(e) = editor {
                        e.write().unwrap().goto(c);
                    }
                    *select = true;
                }
            }

            TreeNavResult::Continue
        } else {
            self.cursor = None;
            TreeNavResult::Exit
        }
    }

    fn goto_home(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let cur_mode = ce.get_cursor().leaf_mode;
                    let depth = ce.get_cursor().tree_addr.len();

                    if depth > 0 {
                        return match ce.goto_home() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *select = false;

                                match self.pxev() {
                                    TreeNavResult::Exit => TreeNavResult::Exit,
                                    TreeNavResult::Continue => {
                                        for _x in 1..depth {
                                            self.dn();
                                            self.goto_end();
                                        }
                                        self.dn();
                                        self.set_leaf_mode(cur_mode);

                                        TreeNavResult::Continue
                                    }
                                }
                            },
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *select = false;
                if c != 0 {
                    self.cursor = Some(0);
                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                        *select = true;
                    }
                    return TreeNavResult::Continue;
                }
            }
        }
        self.cursor = None;
        TreeNavResult::Exit
    }

    fn goto_end(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let cur_mode = ce.get_cursor().leaf_mode;
                    let depth = ce.get_cursor().tree_addr.len();

                    if depth > 0 {
                        return match ce.goto_end() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *select = false;

                                match self.nexd() {
                                    TreeNavResult::Exit => TreeNavResult::Exit,
                                    TreeNavResult::Continue => {
                                        for _x in 1..depth {
                                            self.dn();
                                        }

                                        self.dn();
                                        self.set_leaf_mode(cur_mode);
                                        self.goto_end();

                                        TreeNavResult::Continue
                                    }
                                }
                            },
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *select = false;
                if c < self.n_indices.len()-1 {
                    self.cursor = Some(self.n_indices.len()-1);
                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                        *select = true;
                    }
                    return TreeNavResult::Continue;
                }
            }
        }
        self.cursor = None;
        TreeNavResult::Exit
    }

    fn pxev(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let depth = ce.get_cursor().tree_addr.len();
                    let cur_mode = ce.get_cursor().leaf_mode;

                    if depth > 0 {
                        return match ce.pxev() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *select = false;

                                if c > 0 {
                                    self.cursor = Some(c-1);
                                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                                        *select = true;
                                    }

                                    for _x in 1..depth {
                                        self.dn();
                                        self.goto_end();
                                    }

                                    self.dn();
                                    self.set_leaf_mode(cur_mode);
                                    self.goto_end();

                                    TreeNavResult::Continue                                
                                } else {
                                    TreeNavResult::Exit
                                }
                            }
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *select = false;
                if c > 0 {
                    self.cursor = Some(c-1);
                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                        *select = true;
                    }
                    return TreeNavResult::Continue;
                }
            }
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn nexd(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor.clone() {
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    let mut ce = e.write().unwrap();

                    let depth = ce.get_cursor().tree_addr.len();
                    let cur_mode = ce.get_cursor().leaf_mode;

                    if depth > 0 {
                        return match ce.nexd() {
                            TreeNavResult::Exit => {
                                drop(ce);
                                *select = false;

                                if c+1 < self.n_indices.len() {
                                    self.cursor = Some(c+1);
                                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                                        *select = true;
                                    }

                                    for _x in 1..depth {
                                        self.dn();
                                    }

                                    self.dn();
                                    self.set_leaf_mode(cur_mode);

                                    TreeNavResult::Continue
                                } else {
                                    self.cursor = None;
                                    TreeNavResult::Exit
                                }
                            }
                            TreeNavResult::Continue => TreeNavResult::Continue
                        };
                    }
                }

                *select = false;
                if c+1 < self.n_indices.len() {
                    self.cursor = Some(c+1);
                    if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                        *select = true;
                    }

                    return TreeNavResult::Continue;
                }
            }
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn up(&mut self) -> TreeNavResult {
        if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
            if let Some(e) = editor {
                let mut ce = e.write().unwrap();
                if ce.get_cursor().tree_addr.len() > 0 {
                    ce.up();
                    return TreeNavResult::Continue;
                }
            }
            *select = false;
        }

        self.cursor = None;
        TreeNavResult::Exit
    }

    fn dn(&mut self) -> TreeNavResult {
        if let Some(c) = self.cursor {
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_editor_element_mut(c).as_deref_mut() {
                if let Some(e) = editor {
                    e.write().unwrap().dn();
                } else {
                    let e = make_editor(self.ctx.clone(), t, self.depth+1);
                    e.write().unwrap().goto_home();
                    *editor = Some(e);
                }
            }
        } else {
            self.cursor = Some(0);
            if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
                *select = true;
            }
        }

        TreeNavResult::Continue
    }
}

impl TerminalEditor for ProductEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.el_view_port.to_grid_horizontal().flatten()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        if let Some(ProductEditorElement::N{ t, editor, select }) = self.get_cur_element_mut().as_deref_mut() {
            *select = true;
            if let Some(e) = editor.clone() {
                match e.clone().write().unwrap().handle_terminal_event(event) {
                    TerminalEditorResult::Exit =>
                        match event {
                            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                                *editor = None;
                                TerminalEditorResult::Continue
                            }
                            _ => {
                                drop(e);
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
                x
            }
        } else {
            TerminalEditorResult::Exit
        }
    }
}

impl TerminalTreeEditor for ProductEditor {}

