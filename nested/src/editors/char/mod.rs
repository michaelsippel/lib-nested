use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
        },
        buffer::singleton::*
    },
    crate::{
        type_system::{Context, ReprTree, TypeTerm},
        terminal::{TerminalAtom},
        tree::{NestedNode, TreeNavResult},
        commander::{ObjCommander}
    },
    std::sync::Arc,
    std::sync::RwLock
};

pub fn init_ctx( ctx: &mut Context ) {
    ctx.add_node_ctor(
        "Char",
        Arc::new(|ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
            Some(CharEditor::new_node(ctx))
        }));
}

pub struct CharEditor {
    ctx: Arc<RwLock<Context>>,
    data: SingletonBuffer<char>
}

impl ObjCommander for CharEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) -> TreeNavResult {
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        if cmd_type == (&self.ctx, "( Char )").into() {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let value = cmd_view.get();

                if self.ctx.read().unwrap().meta_chars.contains(&value) {
                    TreeNavResult::Exit
                } else {
                    self.data.set(value);
                    TreeNavResult::Continue
                }
            } else {
                TreeNavResult::Exit
            }
        } else {
            TreeNavResult::Exit   
        }
    }
}

impl CharEditor {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        CharEditor {
            ctx,
            data: SingletonBuffer::new('\0')
        }
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = char>> {
        self.data.get_port()
    }

    pub fn get(&self) -> char {
        self.get_port().get_view().unwrap().get()
    }

    pub fn new_node(ctx0: Arc<RwLock<Context>>) -> NestedNode {
        let data = SingletonBuffer::new('\0');
        let ctx = ctx0.clone();
        let editor = Arc::new(RwLock::new(CharEditor{ ctx, data: data.clone() }));

        NestedNode::new(
            ctx0.clone(),
            ReprTree::new_leaf(
                ctx0.read().unwrap().type_term_from_str("( Char )").unwrap(),
                data.get_port().into()
            ),
            0 // fixme
        )
            .set_view(data
                      .get_port()
                      .map(move |c| TerminalAtom::from(c))
                      .to_grid()
            )
            .set_cmd( editor.clone() )
            .set_editor( editor.clone() )
    }
}
/*
use crate::StringGen;
impl StringGen for CharEditor {
    fn get_string(&self)  -> String {
        String::from(self.get())
    }
}
*/
