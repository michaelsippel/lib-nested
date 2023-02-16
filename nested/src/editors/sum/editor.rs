use {
    r3vi::{
        view::{
            ViewPort, OuterViewPort,
            sequence::*,            
        }
    },
    crate::{
        terminal::{
            TerminalEditor, TerminalEditorResult,
            TerminalEvent, TerminalView
        },
        type_system::{Context},
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        tree::NestedNode,
        commander::Commander,
        PtySegment
    },
    cgmath::{Vector2},
    std::sync::{Arc, RwLock},
    termion::event::{Key}
};

pub struct SumEditor {
    cur: usize,
    pub editors: Vec< NestedNode >,

    port: ViewPort< dyn TerminalView >,
    diag_port: ViewPort< dyn SequenceView<Item = Message> >
}

impl SumEditor {
    pub fn new(
        editors: Vec< NestedNode >
    ) -> Self {
        let port = ViewPort::new();

        SumEditor {
            cur: 0,
            editors,
            port,
            diag_port: ViewPort::new()
        }
    }

    pub fn into_node(self, ctx: Arc<RwLock<Context>>) -> NestedNode {
        let view = self.pty_view();
        let editor = Arc::new(RwLock::new(self));
        NestedNode::new()
            .set_ctx(ctx)
            .set_view(view)
            .set_cmd(editor.clone())
            .set_nav(editor.clone())
//            .set_diag(editor.read().unwrap().diag.clone())
    }

    pub fn get(&self) -> NestedNode {
        self.editors[ self.cur ].clone()
    }

    pub fn select(&mut self, idx: usize) {
        self.cur = idx;

        let tv = self.editors[ self.cur ].get_term_view();
        tv.add_observer( self.port.get_cast() );
        self.port.update_hooks.write().unwrap().clear();
        self.port.add_update_hook( Arc::new(tv.0.clone()) );
        self.port.set_view( Some(tv.get_view_arc()) );

        let dv = self.editors[ self.cur ].get_msg_port();
        dv.add_observer( self.diag_port.get_cast() );
        self.diag_port.update_hooks.write().unwrap().clear();
        self.diag_port.add_update_hook( Arc::new(dv.0.clone()) );
        self.diag_port.set_view( Some(dv.get_view_arc()) );
    }
}

impl TreeNav for SumEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.editors[ self.cur ].get_cursor()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        self.editors[ self.cur ].get_cursor_warp()
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        self.editors[ self.cur ].goby( direction )
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        self.editors[ self.cur ].goto( new_cursor )
    }
}

impl PtySegment for SumEditor {
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.port.outer()
    }
}

impl Commander for SumEditor {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, event: &TerminalEvent) {
        match event {
            TerminalEvent::Input( termion::event::Event::Key(Key::Ctrl('x')) ) => {
                let res = self.editors[ self.cur ].handle_terminal_event( event );
                match res {
                    TerminalEditorResult::Exit => {
                        self.select( (self.cur + 1) % self.editors.len() );
                        if self.editors[ self.cur ].get_cursor().tree_addr.len() == 0 {
                            self.dn();
                        }
                    },
                    _ => {}
                }
            },
            event => {
                self.editors[ self.cur ].handle_terminal_event( event );
            }
        }
    }
}
