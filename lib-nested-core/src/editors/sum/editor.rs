use {
    r3vi::{
        view::{
            ViewPort, OuterViewPort,
            singleton::*,
            sequence::*,            
        }
    },
    laddertypes::{TypeTerm},
    crate::{
        editors::{list::ListCursorMode, ObjCommander},
        reprTree::{Context, ReprTree},
        editTree::{TreeNav, TreeCursor, TreeNavResult, diagnostics::{Diagnostics, Message}, NestedNode},
    },
    cgmath::{Vector2},
    std::sync::{Arc, RwLock}
};

pub struct SumEditor {
    cur: usize,
    pub editors: Vec< NestedNode >,

    addr_port: ViewPort< dyn SequenceView<Item = isize> >,
    mode_port: ViewPort< dyn SingletonView<Item = ListCursorMode> >,

//    port: ViewPort< dyn TerminalView >,
    diag_port: ViewPort< dyn SequenceView<Item = Message> >
}

impl SumEditor {
    pub fn new(
        editors: Vec< NestedNode >
    ) -> Self {
//        let port = ViewPort::new();

        SumEditor {
            cur: 0,
            editors,
//            port,
            diag_port: ViewPort::new(),


            addr_port: ViewPort::new(),
            mode_port: ViewPort::new()
        }
    }

    pub fn init_ctx(ctx: &Arc<RwLock<Context>>) {
        ctx.write().unwrap().add_typename("Sum".into());
    }

    pub fn into_node(self, ctx: Arc<RwLock<Context>>) -> NestedNode {
//        let view = self.pty_view();
        let editor = Arc::new(RwLock::new(self));

        NestedNode::new(
            ctx.clone(),
            ReprTree::new_arc(TypeTerm::TypeID(ctx.read().unwrap().get_typeid("Sum").unwrap())),
            r3vi::buffer::singleton::SingletonBuffer::new(0).get_port()
        )
//            .set_view(view)
            .set_editor(editor.clone())
            .set_cmd(editor.clone())
            .set_nav(editor.clone())
//            .set_diag(editor.read().unwrap().diag.clone())
    }

    pub fn get(&self) -> NestedNode {
        self.editors[ self.cur ].clone()
    }

    pub fn select(&mut self, idx: usize) {
        self.cur = idx;
/* FIXME

        let tv = self.editors[ self.cur ].get_view();
        tv.add_observer( self.port.get_cast() );
        self.port.update_hooks.write().unwrap().clear();
        self.port.add_update_hook( Arc::new(tv.0.clone()) );
        self.port.set_view( Some(tv.get_view_arc()) );

        let dv = self.editors[ self.cur ].get_msg_port();
        dv.add_observer( self.diag_port.get_cast() );
        self.diag_port.update_hooks.write().unwrap().clear();
        self.diag_port.add_update_hook( Arc::new(dv.0.clone()) );
        self.diag_port.set_view( Some(dv.get_view_arc()) );

        let dv = self.editors[ self.cur ].get_addr_view();
        dv.add_observer( self.addr_port.get_cast() );
        self.addr_port.update_hooks.write().unwrap().clear();
        self.addr_port.add_update_hook( Arc::new(dv.0.clone()) );
        self.addr_port.set_view( Some(dv.get_view_arc()) );
        
        let dv = self.editors[ self.cur ].get_mode_view();
        dv.add_observer( self.mode_port.get_cast() );
        self.mode_port.update_hooks.write().unwrap().clear();
        self.mode_port.add_update_hook( Arc::new(dv.0.clone()) );
        self.mode_port.set_view( Some(dv.get_view_arc()) );
  */
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

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        self.addr_port.outer()
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        self.mode_port.outer()
    }
}

impl ObjCommander for SumEditor {
    fn send_cmd_obj(&mut self, obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        self.editors[ self.cur ].send_cmd_obj( obj )
    }
}
