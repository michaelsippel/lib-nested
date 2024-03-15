
use {
    laddertypes::TypeTerm,
    r3vi::{
        view::{OuterViewPort,singleton::*},
        buffer::{singleton::*, vec::*}
    },
    crate::{
        repr_tree::{ReprTree, Context},
        edit_tree::{
            EditTree,
            diagnostics::Message
        }
    },

    std::sync::{Arc, RwLock}
};


pub struct DigitEditor {
    pub(super) ctx: Arc<RwLock<Context>>,
    pub(super) radix: u32,
    pub(super) data: SingletonBuffer<char>,
    pub(super) msg: VecBuffer<Message>,
}


impl DigitEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, radix: u32, data: SingletonBuffer<char>) -> Self {
        DigitEditor {
            ctx,
            radix,
            data,
            msg: VecBuffer::new(),
        }
    }

    pub fn into_node(self, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> EditTree { 
      //  let data = self.get_data();        
        let editor = Arc::new(RwLock::new(self));
        let ed = editor.write().unwrap();
        let r = ed.radix;

        EditTree::new(ed.ctx.clone(), depth)
            .set_editor(editor.clone())
            .set_cmd(editor.clone())
            .set_diag(
                ed.msg.get_port().to_sequence()
            )
    }

    pub fn attach_to(&mut self, source: OuterViewPort<dyn SingletonView<Item = u32>>) {
        /*
        source.add_observer(
            Arc::new(NotifyFnObserver::new(|_msg| {
                self.data.set( source.get() )
            }))
        );
        */
    }

    pub fn get_data_port(&self) -> OuterViewPort<dyn SingletonView<Item = Result<u32, char>>> {
        let radix = self.radix;
        self.data.get_port().map(move |c|
            if let Some(d) = c.to_digit(radix) {
                Ok(d)
            } else {
                Err(c)
            }
        )
    }

    pub fn get_type(&self) -> TypeTerm {
        TypeTerm::TypeID(self.ctx.read().unwrap().get_typeid("Digit").unwrap())
    }
/*
    pub fn get_data(&self) -> Arc<RwLock<ReprTree>> {
        ReprTree::ascend(
            &ReprTree::from_view(
                self.ctx.read().unwrap().type_term_from_str("<Seq u32>").unwrap(),
                self.get_data_port()
            ),
            self.get_type()
        )
    }
    */
}


