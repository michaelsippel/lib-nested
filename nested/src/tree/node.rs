use {
    std::{sync::{Arc, RwLock}, any::Any},
    cgmath::{Vector2, Point2},
    r3vi::{
        view::{View, ViewPort, OuterViewPort, AnyOuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*}
    },
    crate::{
        type_system::{ReprTree, Context, TypeTerm},
        terminal::{TerminalView, TerminalEvent, TerminalEditor, TerminalEditorResult, TerminalAtom},
        diagnostics::{Diagnostics, Message},
        tree::{TreeNav, TreeCursor, TreeNavResult},
        editors::list::{ListCursorMode},
        commander::ObjCommander,
    }
};

#[derive(Clone)]
pub struct NestedNode {
    /// depth
    pub depth: usize,
    
    /// context
    pub ctx: Option<Arc<RwLock<Context>>>,

    /// abstract editor
    pub editor: Option<Arc<dyn Any + Send + Sync>>,

    /// abstract data view
    pub data: Option<Arc<RwLock<ReprTree>>>,

    /// display view
    pub view: Option<OuterViewPort<dyn TerminalView>>,

    /// diagnostics
    pub diag: Option<OuterViewPort<dyn SequenceView<Item = Message>>>,

    /// commander
    pub cmd: Option<Arc<RwLock<dyn ObjCommander + Send + Sync>>>,

    /// tree navigation
    pub tree_nav: Option<Arc<RwLock<dyn TreeNav + Send + Sync>>>,
}

impl ObjCommander for NestedNode {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        if let Some(cmd) = self.cmd.as_ref() {
            // todo: filter out tree-nav cmds and send them to tree_nav
            cmd.write().unwrap().send_cmd_obj(cmd_obj);
        }
    }
}

/*
impl TreeType for NestedNode {
    fn get_type(&self, addr: &TreeAddr) -> TypeLadder {
        if let Some(editor) = self.editor {
            editor.read().unwrap().get_type(addr)
        } else {
            vec![]
        }
    }
}
*/
// todo: remove that at some point
impl TerminalEditor for NestedNode {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        let buf = SingletonBuffer::new(event.clone());

        if let (Some(cmd),Some(ctx)) = (self.cmd.as_ref(),self.ctx.as_ref()) {
            cmd.write().unwrap().send_cmd_obj(
                ReprTree::new_leaf(
                    ctx.read().unwrap().type_term_from_str("( TerminalEvent )").unwrap(),
                    AnyOuterViewPort::from(buf.get_port())
                ));
        }

        TerminalEditorResult::Continue
    }
}

impl TreeNav for NestedNode {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.read().unwrap().get_cursor()
        } else {
            TreeCursor::default()
        }
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.read().unwrap().get_addr_view()
        } else {
            OuterViewPort::default()
        }
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.read().unwrap().get_mode_view()
        } else {
            OuterViewPort::default()
        }        
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.read().unwrap().get_cursor_warp()
        } else {
            TreeCursor::default()
        }
    }

    fn get_max_depth(&self) -> usize {
        0
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.write().unwrap().goby(direction)
        } else {
            TreeNavResult::Exit
        }
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        if let Some(tn) = self.tree_nav.as_ref() {
            tn.write().unwrap().goto(new_cursor)
        } else {
            TreeNavResult::Exit
        }
    }
}

impl Diagnostics for NestedNode {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.get_diag()
    }
}

impl NestedNode {
    pub fn new(depth: usize) -> Self {
        NestedNode {
            depth,
            ctx: None,
            data: None,
            editor: None,
            view: None,
            diag: None,
            cmd: None,
            tree_nav: None
        }
    }

    pub fn from_char(ctx: Arc<RwLock<Context>>, c: char) -> NestedNode {
        let buf = r3vi::buffer::singleton::SingletonBuffer::<char>::new(c);

        NestedNode::new(0)
            .set_view(buf.get_port()
                      .map(|c| TerminalAtom::from(c))
                      .to_index()
                      .map_key(
                          |x| {
                              Point2::new(0, 0)
                          },
                          |p| {
                              if *p == Point2::new(0,0) { Some(()) } else { None }
                          })
            )
            .set_data(
                ReprTree::new_leaf(
                    (&ctx, "( Char )"),
                    buf.get_port().into()
                )
            )
            .set_editor(Arc::new(RwLock::new(buf)))
            .set_ctx(ctx)
    }

    pub fn set_ctx(mut self, ctx: Arc<RwLock<Context>>) -> Self {
        self.ctx = Some(ctx);
        self
    }

    pub fn set_data(mut self, data: Arc<RwLock<ReprTree>>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn set_editor(mut self, editor: Arc<dyn Any + Send + Sync>) -> Self {
        self.editor = Some(editor);
        self
    }

    pub fn set_view(mut self, view: OuterViewPort<dyn TerminalView>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn set_cmd(mut self, cmd: Arc<RwLock<dyn ObjCommander + Send + Sync>>) -> Self {
        self.cmd = Some(cmd);
        self
    }

    pub fn set_nav(mut self, nav: Arc<RwLock<dyn TreeNav + Send + Sync>>) -> Self {
        self.tree_nav = Some(nav);
        self
    }

    pub fn set_diag(mut self, diag: OuterViewPort<dyn SequenceView<Item = Message>>) -> Self {
        self.diag = Some(diag);
        self
    }

    pub fn get_diag(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.diag.clone().unwrap_or(ViewPort::new().into_outer())
    }
    
    pub fn get_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.view.clone().unwrap_or(ViewPort::new().into_outer())
    }

    pub fn morph(self, ty: TypeTerm) -> NestedNode {
        Context::morph_node(self, ty)
    }

    pub fn get_data_view<'a, V: View + ?Sized + 'static>(&'a self, type_str: impl Iterator<Item = &'a str>) -> Option<Arc<V>>
    where V::Msg: Clone {
        if let Some(ctx) = self.ctx.clone() {
            if let Some(data) = self.data.clone() {
                let type_ladder = type_str.map(|s| ((&ctx, s)).into());

                let repr_tree = ReprTree::descend_ladder(&data, type_ladder)?;
                repr_tree.clone().read().unwrap()
                    .get_view::<V>().clone()
            } else {
                eprintln!("get_data(): no data port");
                None
            }
        } else {
            eprintln!("get_data(): no ctx");
            None
        }
    }

    pub fn get_edit<T: Send + Sync + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        if let Some(edit) = self.editor.clone() {
            if let Ok(edit) = edit.downcast::<RwLock<T>>() {
                Some(edit)
            } else {
                None
            }
        } else {
            None
        }
    }
}

