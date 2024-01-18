use {
    std::{sync::{Arc, RwLock}, any::Any},
    cgmath::{Vector2, Point2},
    r3vi::{
        view::{View, ViewPort, OuterViewPort, AnyOuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*, vec::*}
    },
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{ReprTree, Context},
        edit_tree::{TreeNav, TreeCursor, TreeNavResult, TreeHeightOp, diagnostics::{Diagnostics, Message}},
        editors::{list::{ListCursorMode}, ObjCommander}
    }
};

#[derive(Clone)]
pub struct NestedNodeDisplay {
    /// display view
    pub view: Arc<RwLock<ReprTree>>,

    /// diagnostics
    pub diag: Option< OuterViewPort<dyn SequenceView<Item = Message>> >,

    /// depth
    pub depth: OuterViewPort<dyn SingletonView<Item = usize>>,
}

#[derive(Clone)]
pub struct NestedNodeEdit {
    /// abstract editor
    pub editor: SingletonBuffer<
                    Option< Arc<dyn Any + Send + Sync> >
                >,

    pub spillbuf: Arc<RwLock< Vec< Arc<RwLock< NestedNode >> > >>,

    /// commander & navigation
    pub cmd: SingletonBuffer<
                 Option< Arc<RwLock<dyn ObjCommander + Send + Sync>> >
             >,    /// abstract data view

    pub close_char: SingletonBuffer< Option< char > >,

    // could be replaced by cmd when TreeNav -CmdObjects are used
    pub tree_nav: SingletonBuffer<
                      Option< Arc<RwLock<dyn TreeNav + Send + Sync>> >
                  >,    
}

#[derive(Clone)]
pub struct NestedNode {
    /// context
    pub ctx: Arc<RwLock<Context>>,

    /// viewports for terminal display
    pub disp: NestedNodeDisplay,

    /// editor & commander objects
    pub edit: NestedNodeEdit
}

impl NestedNode {
    pub fn new(ctx: Arc<RwLock<Context>>, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> Self {
        NestedNode {
            disp: NestedNodeDisplay {
                view: ReprTree::new_arc(Context::parse(&ctx, "Display")),
                diag: None,
                depth,
            },
            edit: NestedNodeEdit {
                editor: SingletonBuffer::new(None),
                spillbuf: Arc::new(RwLock::new(Vec::new())),
                cmd: SingletonBuffer::new(None),
                close_char: SingletonBuffer::new(None),            
                tree_nav: SingletonBuffer::new(None),
            },
            ctx
        }
    }
   
    pub fn set_editor(mut self, editor: Arc<dyn Any + Send + Sync>) -> Self {
        self.edit.editor.set(Some(editor));
        self
    }

    pub fn set_cmd(mut self, cmd: Arc<RwLock<dyn ObjCommander + Send + Sync>>) -> Self {
        self.edit.cmd.set(Some(cmd));
        self
    }

    pub fn set_nav(mut self, nav: Arc<RwLock<dyn TreeNav + Send + Sync>>) -> Self {
        self.edit.tree_nav.set(Some(nav));
        self
    }

    pub fn set_diag(mut self, diag: OuterViewPort<dyn SequenceView<Item = Message>>) -> Self {
        self.disp.diag = Some(diag);
        self
    }

    //\\//\\

    pub fn get_diag(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.disp.diag.clone().unwrap_or(ViewPort::new().into_outer())
    }

    pub fn get_edit<T: Send + Sync + 'static>(&self) -> Option<Arc<RwLock<T>>> {
        if let Some(edit) = self.edit.editor.get() {
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

impl TreeNav for NestedNode {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.read().unwrap().get_cursor()
        } else {
            TreeCursor::default()
        }
    }

    fn get_addr_view(&self) -> OuterViewPort<dyn SequenceView<Item = isize>> {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.read().unwrap().get_addr_view()
        } else {
            OuterViewPort::default()
        }
    }

    fn get_mode_view(&self) -> OuterViewPort<dyn SingletonView<Item = ListCursorMode>> {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.read().unwrap().get_mode_view()
        } else {
            OuterViewPort::default()
        }        
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.read().unwrap().get_cursor_warp()
        } else {
            TreeCursor::default()
        }
    }

    fn get_height(&self, op: &TreeHeightOp) -> usize {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.read().unwrap().get_height( op )
        } else {
            0
        }
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.write().unwrap().goby(direction)
        } else {
            TreeNavResult::Exit
        }
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        if let Some(tn) = self.edit.tree_nav.get() {
            tn.write().unwrap().goto(new_cursor)
        } else {
            TreeNavResult::Exit
        }
    }
}

use crate::edit_tree::nav::TreeNavCmd;

impl ObjCommander for NestedNode {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {

        if cmd_obj.read().unwrap().get_type() == &Context::parse(&self.ctx, "TreeNavCmd") {
            if let Some(cmd) = cmd_obj.read().unwrap().get_view::<dyn SingletonView<Item = TreeNavCmd>>() {
                match cmd.get() {
                    TreeNavCmd::pxev => self.pxev(),
                    TreeNavCmd::nexd => self.nexd(),
                    TreeNavCmd::qpxev => self.qpxev(),
                    TreeNavCmd::qnexd => self.qnexd(),

                    TreeNavCmd::up => self.up(),
                    TreeNavCmd::dn => self.dn(),

                    _ => TreeNavResult::Continue
                }
            } else {
                TreeNavResult::Exit
            }
        } else if let Some(cmd) = self.edit.cmd.get() {
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

