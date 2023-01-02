use {
    crate::{
        core::{OuterViewPort},
        type_system::{Context},
        terminal::{TerminalEvent, TerminalView, TerminalEditor, TerminalEditorResult},
        sequence::{SequenceView, decorator::SeqDecorStyle},
        list::{PTYListEditor},
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        sum::SumEditor,
        char_editor::CharEditor,
        integer::PosIntEditor,
        tree::NestedNode,
        Commander, PtySegment
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
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( List TypeTerm 1 )").unwrap(), depth + 1).unwrap(),
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( PosInt 10 )").unwrap(), depth + 1 ).unwrap(),
                    Context::make_editor( &ctx, ctx.read().unwrap().type_term_from_str("( Symbol )").unwrap(), depth + 1 ).unwrap()
                ])))
        }
    }

    pub fn into_node(self) -> NestedNode {
        NestedNode::new()
            .set_ctx(self.ctx.clone())
            .set_nav(self.sum_edit.clone())
            .set_cmd(self.sum_edit.clone())
            .set_view(
                self.sum_edit.read().unwrap().pty_view()
            )
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
                        if *c  == '(' {
                            let _child = Arc::new(RwLock::new(TypeTermEditor {
                                ctx: self.ctx.clone(),
                                ty: self.ty.clone(),
                                sum_edit: Arc::new(RwLock::new(SumEditor::new(
                                    vec![
                                        self.sum_edit.read().unwrap().editors[0].clone(),
                                        self.sum_edit.read().unwrap().editors[1].clone(),
                                        self.sum_edit.read().unwrap().editors[2].clone(),
                                    ])))
                            }));
                            self.ty = TypeTermVar::List;
                            self.sum_edit.write().unwrap().select(0);
/*
                            let l = self.node.editors[0].clone();
                            let l = l.downcast::<RwLock<PTYListEditor<TypeTermEditor>>>().unwrap();
                            l.write().unwrap().data.push(child);
                            */
                        } else {
                            self.sum_edit.write().unwrap().send_cmd( event );
                        }
                    }
                }
            },
            event => {
                self.sum_edit.write().unwrap().send_cmd( event );
            }
        }
    }
}

