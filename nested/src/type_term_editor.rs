use {
    crate::{
        core::{OuterViewPort, Context},
        terminal::{TerminalEvent, TerminalView, TerminalEditor, TerminalEditorResult},
        sequence::{SequenceView, decorator::SeqDecorStyle},
        list::{PTYListEditor},
        tree::{TreeNav, TreeCursor, TreeNavResult},
        diagnostics::{Diagnostics, Message},
        sum::SumEditor,
        char_editor::CharEditor,
        integer::PosIntEditor,
        Nested
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
    ty: TypeTermVar,
    node: SumEditor,
}

impl TypeTermEditor {
    pub fn new(ctx: Arc<RwLock<Context>>, depth: usize) -> Self {
        TypeTermEditor {
            ty: TypeTermVar::Any,
            node: SumEditor::new(
                vec![
                    Arc::new(RwLock::new(PTYListEditor::new(
                        Box::new({
                            let ctx = ctx.clone();
                            move || {
                                Arc::new(RwLock::new(TypeTermEditor::new(ctx.clone(), depth+1)))
                            }
                        }),
                        SeqDecorStyle::HorizontalSexpr,
                        Some(' '),
                        depth
                    ))),
                    Arc::new(RwLock::new(PosIntEditor::new(10))),
                    Arc::new(RwLock::new(PTYListEditor::new(
                        Box::new({
                            let ctx = ctx.clone();
                            move || {
                                Arc::new(RwLock::new(CharEditor::new_node(&ctx)))
                            }
                        }),
                        SeqDecorStyle::Plain,
                        None,
                        depth
                    ))),
                ])
        }
    }
}

impl TreeNav for TypeTermEditor {
    fn get_cursor(&self) -> TreeCursor {
        self.node.get_cursor()
    }

    fn get_cursor_warp(&self) -> TreeCursor {
        self.node.get_cursor_warp()
    }

    fn goby(&mut self, direction: Vector2<isize>) -> TreeNavResult {
        self.node.goby( direction )
    }

    fn goto(&mut self, new_cursor: TreeCursor) -> TreeNavResult {
        self.node.goto( new_cursor )
    }
}

impl TerminalEditor for TypeTermEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.node.get_term_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input( termion::event::Event::Key(Key::Char(c)) ) => {
                match self.ty {
                    TypeTermVar::Any => {
                        self.ty =
                            if *c == '(' {
                                self.node.select(0);
                                self.dn();
                                TypeTermVar::List
                            } else if c.to_digit(10).is_some() {
                                self.node.select(1);
                                self.dn();
                                self.node.handle_terminal_event( event );
                                TypeTermVar::Num
                            } else {
                                self.node.select(2);
                                self.dn();
                                self.node.handle_terminal_event( event );
                                TypeTermVar::Symbol
                            };
                        TerminalEditorResult::Continue
                    },
                    _ => {
                        if *c  == '(' {
                            let _child = Arc::new(RwLock::new(TypeTermEditor {
                                ty: self.ty.clone(),
                                node: SumEditor::new(
                                    vec![
                                        self.node.editors[0].clone(),
                                        self.node.editors[1].clone(),
                                        self.node.editors[2].clone(),
                                    ])
                            }));

                            self.ty = TypeTermVar::List;
                            self.node.select(0);
/*
                            let l = self.node.editors[0].clone();
                            let l = l.downcast::<RwLock<PTYListEditor<TypeTermEditor>>>().unwrap();
                            l.write().unwrap().data.push(child);
                            */
                            TerminalEditorResult::Continue
                        } else {
                            self.node.handle_terminal_event( event )
                        }
                    }
                }
            },
            event => {
                self.node.handle_terminal_event( event )
            }
        }
    }
}

impl Diagnostics for TypeTermEditor {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = Message>> {
        self.node.get_msg_port()
    }
}

impl Nested for TypeTermEditor {}

