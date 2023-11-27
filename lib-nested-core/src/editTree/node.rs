use {
    std::{sync::{Arc, RwLock}, any::Any},
    cgmath::{Vector2, Point2},
    r3vi::{
        view::{View, ViewPort, OuterViewPort, AnyOuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*, vec::*}
    },
    laddertypes::{TypeTerm},
    crate::{
        reprTree::{ReprTree, Context},
        editTree::{TreeNav, TreeCursor, TreeNavResult, TreeHeightOp, diagnostics::{Diagnostics, Message}},
        editors::{list::{ListCursorMode}, ObjCommander}
    }
};

//* TODO: refactoring proposal
/*

struct NestedNodeDisplay {
    /// display view
    pub view: Option< Arc<RwLock<ReprTree>> >,

    /// diagnostics
    pub diag: Option< OuterViewPort<dyn SequenceView<Item = diagnostics::Message>> >,

    /// depth
    pub depth: SingletonBuffer< usize >,
}

struct NestedNodeEdit {
    /// abstract editor
    pub editor: SingletonBuffer<
                    Option< Arc<dyn Any + Send + Sync> >
    >,

    pub spillbuf: VecBuffer< NestedNode >,

    /// commander & navigation
    pub cmd: SingletonBuffer<
                 Option< Arc<RwLock<dyn ObjCommander + Send + Sync>> >
             >,    /// abstract data view
    pub data: Arc<RwLock<ReprTree>>,

    pub close_char: SingletonBuffer< Option< char > >,

    // could be replaced by cmd when TreeNav -CmdObjects are used
    pub tree_nav: SingletonBuffer<
                      Option< Arc<RwLock<dyn TreeNav + Send + Sync>> >
                  >,    
}

pub struct NewNestedNode {
    /// context
    pub ctx: Arc<RwLock<Context>>,

    /// abstract data view
    pub data: Arc<RwLock<ReprTree>>,

    /// viewports for terminal display
    pub disp: NestedNodeDisplay,

    /// editor & commander objects
    pub edit: NestedNodeEdit
}
*/

#[derive(Clone)]
pub struct NestedNode {    
    /// context
    pub ctx: Arc<RwLock<Context>>,

    /// abstract data view
    pub data: Arc<RwLock<ReprTree>>,

    /// display view
    pub view: Option< Arc<RwLock<ReprTree>> >,

    /// diagnostics
    pub diag: Option< OuterViewPort<dyn SequenceView<Item = Message>> >,

    /// depth
    pub depth: OuterViewPort< dyn SingletonView<Item = usize> >,

    /// abstract editor
    pub editor: SingletonBuffer<
                    Option< Arc<dyn Any + Send + Sync> >
                >,

    pub spillbuf: Arc<RwLock<Vec<Arc<RwLock<NestedNode>>>>>,

    /// commander & navigation
    pub cmd: SingletonBuffer<
                 Option< Arc<RwLock<dyn ObjCommander + Send + Sync>> >
             >,
    pub close_char: SingletonBuffer<
                        Option< char >
                    >,
    pub tree_nav: SingletonBuffer<
                      Option< Arc<RwLock<dyn TreeNav + Send + Sync>> >
                  >,
}

impl NestedNode {
    pub fn new(ctx: Arc<RwLock<Context>>, data: Arc<RwLock<ReprTree>>, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> Self {
        NestedNode {
            ctx,
            data,
            view: None,
            diag: None,
            depth,
            editor: SingletonBuffer::new(None),
            spillbuf: Arc::new(RwLock::new(Vec::new())),
            cmd: SingletonBuffer::new(None),
            close_char: SingletonBuffer::new(None),
            tree_nav: SingletonBuffer::new(None)
        }
    }

    /* TODO: move into separate file/module
    */
    pub fn from_char(ctx: Arc<RwLock<Context>>, c: char) -> NestedNode {
        let buf = r3vi::buffer::singleton::SingletonBuffer::<char>::new(c);

        NestedNode::new(
            ctx.clone(),
            ReprTree::new_leaf(
                Context::parse(&ctx, "Char"),
                buf.get_port().into()
            ),
            SingletonBuffer::new(0).get_port()
      )
            /*
            .set_view(buf.get_port()
                      .map(|c| TerminalAtom::from(c))
                      .to_index()
                      .map_key(
                          |_x| {
                              Point2::new(0, 0)
                          },
                          |p| {
                              if *p == Point2::new(0,0) { Some(()) } else { None }
                          })
            )
                */
            .set_editor(Arc::new(RwLock::new(buf)))
    }

    
    //\\//\\

    pub fn morph(self, ty: TypeTerm) -> NestedNode {
        Context::morph_node(self, ty)
    }

    pub fn get_type(&self) -> TypeTerm {
        self.data.read().unwrap().get_type().clone()
    }

    //\\//\\
    
    pub fn set_editor(mut self, editor: Arc<dyn Any + Send + Sync>) -> Self {
        self.editor.set(Some(editor));
        self
    }

    pub fn set_view(mut self, view: Arc<RwLock<ReprTree>>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn set_cmd(mut self, cmd: Arc<RwLock<dyn ObjCommander + Send + Sync>>) -> Self {
        self.cmd.set(Some(cmd));
        self
    }

    pub fn set_nav(mut self, nav: Arc<RwLock<dyn TreeNav + Send + Sync>>) -> Self {
        self.tree_nav.set(Some(nav));
        self
    }

    pub fn set_diag(mut self, diag: OuterViewPort<dyn SequenceView<Item = Message>>) -> Self {
        self.diag = Some(diag);
        self
    }

    //\\//\\

    pub fn get_diag(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.diag.clone().unwrap_or(ViewPort::new().into_outer())
    }

    pub fn get_view(&self) -> Option< Arc<RwLock<ReprTree>> > {
        self.view.clone()
    }
    
    pub fn get_data_port<'a, V: View + ?Sized + 'static>(&'a self, type_str: impl Iterator<Item = &'a str>) -> Option<OuterViewPort<V>>
    where V::Msg: Clone {
        let ctx = self.ctx.clone();
        let type_ladder = type_str.map(|s| Context::parse(&ctx, s));

        let repr_tree = ReprTree::descend_ladder(&self.data, type_ladder)?;
        repr_tree.clone().read().unwrap()
            .get_port::<V>().clone()
    }

    pub fn get_data_view<'a, V: View + ?Sized + 'static>(&'a self, type_str: impl Iterator<Item = &'a str>) -> Option<Arc<V>>
    where V::Msg: Clone {
        self.get_data_port::<V>(type_str)?.get_view()
    }

    /* TODO
    pub fn get_seq_view<'a, T: Clone>(&self, type_str: impl Iterator<Item = &'a str>) -> Option<OuterViewPort<dyn SingletonView<Item = T>>> {
        self.get_data_view::<dyn SequenceView<Item = NestedNode>>(type_str)
            .unwrap()
            .map({
                move |node| {
                    node.get_data_view::<dyn SingletonView<Item = T>>().get()
                }
            })
    }
     */
    
    pub fn get_edit<T: Send + Sync + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        if let Some(edit) = self.editor.get() {
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

/* TODO: remove that at some point
*/
/*
impl TerminalEditor for NestedNode {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.get_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        let buf = SingletonBuffer::new(event.clone());

        if let Some(cmd) = self.cmd.get() {
            cmd.write().unwrap().send_cmd_obj(
                ReprTree::new_leaf(
                    self.ctx.read().unwrap().type_term_from_str("TerminalEvent").unwrap(),
                    AnyOuterViewPort::from(buf.get_port())
                ));
        }

        TerminalEditorResult::Continue
    }
}
*/
impl TreeNav for NestedNode {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(tn) = self.tree_nav.get() {
            tn.read().unwrap().get_cursor()
        } else {
            TreeCursor::default()
        }
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        if let Some(tn) = self.tree_nav.get() {
            tn.read().unwrap().get_addr_view()
        } else {
            OuterViewPort::default()
        }
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        if let Some(tn) = self.tree_nav.get() {
            tn.read().unwrap().get_mode_view()
        } else {
            OuterViewPort::default()
        }        
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        if let Some(tn) = self.tree_nav.get() {
            tn.read().unwrap().get_cursor_warp()
        } else {
            TreeCursor::default()
        }
    }

    fn get_height(&self, op: &TreeHeightOp) -> usize {
        if let Some(tn) = self.tree_nav.get() {
            tn.read().unwrap().get_height( op )
        } else {
            0
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        if let Some(tn) = self.tree_nav.get() {
            tn.write().unwrap().goby(direction)
        } else {
            TreeNavResult::Exit
        }
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        if let Some(tn) = self.tree_nav.get() {
            tn.write().unwrap().goto(new_cursor)
        } else {
            TreeNavResult::Exit
        }
    }
}

impl ObjCommander for NestedNode {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        if let Some(cmd) = self.cmd.get() {
            // todo: filter out tree-nav cmds and send them to tree_nav
            cmd.write().unwrap().send_cmd_obj(cmd_obj)
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

