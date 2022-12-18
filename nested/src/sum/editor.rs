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
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        terminal::{TerminalStyle},
        Nested
    },
    cgmath::{Vector2, Point2},
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
    std::ops::{Deref, DerefMut}
};

pub struct SumEditor {
    cur: usize,
    pub editors: Vec< Arc<RwLock<dyn Nested + Send + Sync + 'static>> >,

    port: ViewPort< dyn TerminalView >,
    diag_port: ViewPort< dyn SequenceView<Item = Message> >
}

impl SumEditor {
    pub fn new(
        editors: Vec< Arc<RwLock<dyn Nested + Send + Sync + 'static>> >
    ) -> Self {
        let port = ViewPort::new();
        //let mut diag_buf = VecBuffer::new();

        SumEditor {
            cur: 0,
            editors,
            port,
            diag_port: ViewPort::new()//diag_buf.get_port().to_sequence()
        }
    }

    pub fn get(&self) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        self.editors[ self.cur ].clone()
    }

    pub fn select(&mut self, idx: usize) {
        self.cur = idx;

        let tv = self.editors[ self.cur ].read().unwrap().get_term_view();
        tv.add_observer( self.port.get_cast() );
        self.port.update_hooks.write().unwrap().clear();
        self.port.add_update_hook( Arc::new(tv.0.clone()) );
        self.port.set_view( Some(tv.get_view_arc()) );

        let dv = self.editors[ self.cur ].read().unwrap().get_msg_port();
        dv.add_observer( self.diag_port.get_cast() );
        self.diag_port.update_hooks.write().unwrap().clear();
        self.diag_port.add_update_hook( Arc::new(dv.0.clone()) );
        self.diag_port.set_view( Some(dv.get_view_arc()) );
    }
}

impl TreeNav for SumEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.editors[ self.cur ].write().unwrap().get_cursor()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        self.editors[ self.cur ].write().unwrap().get_cursor_warp()
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        self.editors[ self.cur ].write().unwrap().goby( direction )
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        self.editors[ self.cur ].write().unwrap().goto( new_cursor )
    }
}

impl TerminalEditor for SumEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.port.outer()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input( termion::event::Event::Key(Key::Ctrl('x')) ) => {
                let res = self.editors[ self.cur ].write().unwrap().handle_terminal_event( event );
                match res {
                    TerminalEditorResult::Exit => {
                        self.select( (self.cur + 1) % self.editors.len() );
                        if self.editors[ self.cur ].read().unwrap().get_cursor().tree_addr.len() == 0 {
                            self.dn();
                        }
                        TerminalEditorResult::Continue
                    },
                    _ => TerminalEditorResult::Continue
                }
            },
            event => {
                self.editors[ self.cur ].write().unwrap().handle_terminal_event( event )
            }
        }
    }
}

impl Diagnostics for SumEditor {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.diag_port.outer()
    }
}

impl Nested for SumEditor {}


