use {
    r3vi::{
        view::{
            OuterViewPort,
            sequence::*
        }
    },
    crate::{
        type_system::{Context},
        terminal::{TerminalEvent, TerminalView, TerminalEditor, TerminalEditorResult},
        editors::{
            list::*,
            sum::*,
            char::CharEditor,
            integer::PosIntEditor,
        },
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        tree::NestedNode,
        commander::Commander,
        PtySegment
    },
    cgmath::{Vector2},
    termion::event::{Key},
    std::{
        sync::{Arc, RwLock}
    }
};

#[derive(Clone)]
enum TypeTermVar {
    Any,
    Symbol,
    Num,
    List
}

pub struct TypeTermEditor {
    ctx: Arc<RwLock<Context>>,
    ty: TypeTermVar,
    sum_edit: Arc<RwLock<SumEditor>>
}

impl TypeTermEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, depth: usize) -> Self {
        TypeTermEditor {
            ctx: ctx.clone(),
            ty: TypeTermVar::Any,
            sum_edit: Arc::new(RwLock::new(SumEditor::new(
                vec![
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( List TypeTerm )").unwrap(), depth + 1).unwrap(),
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( PosInt 10 )").unwrap(), depth + 1 ).unwrap(),
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( Symbol )").unwrap(), depth + 1 ).unwrap()
                ])))
        }
    }

    pub fn into_node(self) -> NestedNode {
        let ctx = self.ctx.clone();
        let sum_edit = self.sum_edit.clone();
        let view = sum_edit.read().unwrap().pty_view();
        let editor = Arc::new(RwLock::new(self));

        NestedNode::new()
            .set_ctx(ctx)
            .set_nav(sum_edit)
            .set_cmd(editor.clone())
            .set_view(view)
    }
}

impl Commander for TypeTermEditor {
    type Cmd = TerminalEvent;
 
    fn send_cmd(&mut self, event: &TerminalEvent) {
        match event {
            TerminalEvent::Input( termion::event::Event::Key(Key::Char(c)) ) => {
                match self.ty {
                    TypeTermVar::Any => {
                        self.ty =
                            if *c == '(' {
                                let mut se = self.sum_edit.write().unwrap();
                                se.select(0);
                                se.dn();
                                TypeTermVar::List
                            } else if c.to_digit(10).is_some() {
                                let mut se = self.sum_edit.write().unwrap();
                                se.select(1);
                                se.dn();
                                se.send_cmd( event );
                                TypeTermVar::Num
                            } else {
                                let mut se = self.sum_edit.write().unwrap();
                                se.select(2);
                                se.dn();
                                se.send_cmd( event );
                                TypeTermVar::Symbol
                            };
                    },
                    _ => {
                        /*
                        if *c  == '(' {
                            let child = TypeTermEditor {
                                ctx: self.ctx.clone(),
                                ty: self.ty.clone(),
                                sum_edit: Arc::new(RwLock::new(SumEditor::new(
                                    vec![
                                        self.sum_edit.read().unwrap().editors[0].clone(),
                                        self.sum_edit.read().unwrap().editors[1].clone(),
                                        self.sum_edit.read().unwrap().editors[2].clone(),
                                    ])))
                            };

                            self.ty = TypeTermVar::List;
                            self.sum_edit.write().unwrap().select(0);

                            let l = self.sum_edit.read().unwrap().editors[0].clone();
                            let l = l.editor.clone().unwrap().downcast::<RwLock<ListEditor>>().unwrap();
                            l.write().unwrap().insert(TypeTermEditor::new(self.ctx.clone(), 1).into_node());
                    } else {
                        */
                        self.sum_edit.write().unwrap().send_cmd( event );
                        //}
                    }
                }
            },
            event => {
                self.sum_edit.write().unwrap().send_cmd( event );
            }
        }
    }
}

