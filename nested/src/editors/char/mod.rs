use {
    r3vi::{
        view::{
            OuterViewPort,
            singleton::*,
        },
        buffer::singleton::*
    },
    crate::{
        type_system::{Context, ReprTree},
        terminal::{TerminalAtom, TerminalStyle},
        tree::NestedNode,
        commander::{ObjCommander}
    },
    std::sync::Arc,
    std::sync::RwLock
};

pub struct CharEditor {
    ctx: Arc<RwLock<Context>>,
    data: SingletonBuffer<Option<char>>
}

impl ObjCommander for CharEditor {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        let cmd_obj = cmd_obj.read().unwrap();
        let cmd_type = cmd_obj.get_type().clone();

        let char_type = (&self.ctx, "( Char )").into();
        //let _term_event_type = (&ctx, "( TerminalEvent )").into();

        if cmd_type == char_type {
            if let Some(cmd_view) = cmd_obj.get_view::<dyn SingletonView<Item = char>>() {
                let value = cmd_view.get();
                self.data.set(Some(value));
            }
        }
/*
        if cmd_type == term_event_type {
            if let Some(te_view) = cmd_obj.get_view::<dyn SingletonView<Item = TerminalEvent>>() {
                let event = te_view.get();
                match event {
                    TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                        self.data.set(Some(c));
                    }

                    TerminalEvent::Input(Event::Key(Key::Backspace))
                        | TerminalEvent::Input(Event::Key(Key::Delete)) => {
                            self.data.set(None);
                        }

                    _ => {}
                }                
            }
    }
        */
    }
}

impl CharEditor {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        CharEditor {
            ctx,
            data: SingletonBuffer::new(None)
        }
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<char>>> {
        self.data.get_port()
    }

    pub fn get(&self) -> char {
        self.get_port().get_view().unwrap().get().unwrap_or('?')
    }

    pub fn new_node(ctx0: Arc<RwLock<Context>>) -> NestedNode {
        let data = SingletonBuffer::new(None);

        let ctx = ctx0.clone();
        
        NestedNode::new(0)
            .set_ctx(ctx0.clone())
            .set_data(
                ReprTree::new_leaf(
                    ctx0.read().unwrap().type_term_from_str("( Char )").unwrap(),
                    data.get_port().into()   
                )
            )
            .set_view(data
                      .get_port()
                      .map(move |c| {
                          match c {
                              Some(c) => TerminalAtom::from(c),
                              None => TerminalAtom::new(' ', TerminalStyle::bg_color((255,0,0)))
                          }
                      })
                      .to_grid()
            )
            .set_cmd(
                Arc::new(RwLock::new(CharEditor{ ctx, data }))
            )
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
