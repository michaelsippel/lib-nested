use {
    std::{
        sync::{Arc, RwLock},
        collections::HashMap
    },
    cgmath::{Vector2, Point2},
    termion::event::{Event, Key},
    nested::{
        vec::VecBuffer,
        list::{ListEditor, PTYListEditor},
        sequence::{SequenceView, decorator::{Separate, Wrap, SeqDecorStyle}},
        core::{TypeTerm, Context},
        core::{OuterViewPort, ViewPort},
        index::{IndexArea, IndexView},
        char_editor::CharEditor,
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView, make_label
        },
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::{Diagnostics},
        product::ProductEditor,
        sum::SumEditor,
        Nested
    }
};

trait Action {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>>;
}

pub struct ActCd {}
impl Action for ActCd {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( Path )").unwrap() ] )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>
    }
}

pub struct ActLs {}
impl Action for ActLs {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(1, 0), " Files")
                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( List Path )").unwrap() ] )
                             .with_t(Point2::new(1, 1), " Options")
                             .with_n(Point2::new(0, 1), vec![ ctx.read().unwrap().type_term_from_str("( List String )").unwrap() ] )

        )) as Arc<RwLock<dyn Nested + Send + Sync>>
    }
}

pub struct ActEcho {}
impl Action for ActEcho {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        
        let a = Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( String )").unwrap() ] )

        )) as Arc<RwLock<dyn Nested + Send + Sync>>;

        let b = Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )

        )) as Arc<RwLock<dyn Nested + Send + Sync>>;

        let mut x = Arc::new(RwLock::new( SumEditor::new(
            vec![
                a, b
            ]
        )  ));

        x.write().unwrap().select(0);
        x
    }
}

pub struct ActCp {}
impl Action for ActCp {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(1, 1), " Source")
                             .with_n(Point2::new(0, 1), vec![ ctx.read().unwrap().type_term_from_str("( Path )").unwrap() ] )
                             .with_t(Point2::new(1, 2), " Destination")
                             .with_n(Point2::new(0, 2), vec![ ctx.read().unwrap().type_term_from_str("( Path )").unwrap() ] )
                             .with_t(Point2::new(1, 3), " Options")
                             .with_n(Point2::new(0, 3), vec![ ctx.read().unwrap().type_term_from_str("( List Symbol )").unwrap() ] )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>
    }
}

pub struct ActNum {}
impl Action for ActNum {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(1, 1), " Value")
                             .with_n(Point2::new(0, 1), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
                             .with_t(Point2::new(1, 2), " Radix")
                             .with_n(Point2::new(0, 2), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 10 BigEndian )").unwrap() ] )

    )) as Arc<RwLock<dyn Nested + Send + Sync>>

//        Arc::new(RwLock::new(nested::integer::PosIntEditor::new(10)))
    }
}

pub struct ActLet {}
impl Action for ActLet {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn Nested + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_n(Point2::new(0, 0), vec![ ctx.read().unwrap().type_term_from_str("( Symbol )").unwrap() ] )
                             .with_t(Point2::new(1, 0), " : ")
                             .with_n(Point2::new(2, 0), vec![ ctx.read().unwrap().type_term_from_str("( TypeTerm )").unwrap() ] )
                             .with_t(Point2::new(3, 0), " := ")
                             .with_n(Point2::new(4, 0), vec![ ctx.read().unwrap().type_term_from_str("( PosInt 16 BigEndian )").unwrap() ] )
        )) as Arc<RwLock<dyn Nested + Send + Sync>>
    }
}

pub struct Commander {
    ctx: Arc<RwLock<Context>>,
    cmds: HashMap<String, Arc<dyn Action + Send + Sync>>,

    valid: Arc<RwLock<bool>>,
    confirmed: bool,
    symbol_editor: PTYListEditor<CharEditor>,
    cmd_editor: Option<Arc<RwLock<dyn Nested + Send + Sync>>>,

    view_elements: VecBuffer<OuterViewPort<dyn TerminalView>>,
    out_port: OuterViewPort<dyn TerminalView>,

    m_buf: VecBuffer<OuterViewPort<dyn SequenceView<Item = nested::diagnostics::Message>>>,
    msg_port: OuterViewPort<dyn SequenceView<Item = nested::diagnostics::Message>>
}

impl Commander {
    pub fn new(ctx: Arc<RwLock<Context>>) -> Self {
        let port = ViewPort::new();
        let mut view_elements = VecBuffer::with_port(port.inner());

        let symbol_editor = PTYListEditor::new(
            || {
                Arc::new(RwLock::new(CharEditor::new()))
            },
            SeqDecorStyle::Plain,
            '\n',
            0
        );

        let valid = Arc::new(RwLock::new(false));
        view_elements.push(symbol_editor
                           .get_term_view()
                           .map_item({
                               let valid = valid.clone();
                               move
                               |pos, mut a| {
                                   if *valid.read().unwrap() {
                                       a.add_style_back(TerminalStyle::fg_color((0,255,0)))
                                   } else {
                                       a.add_style_back(TerminalStyle::fg_color((255,0,0)))
                                   }
                               }
                           }));

        let mut cmds = HashMap::new();

        cmds.insert("let".into(), Arc::new(ActLet{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("cd".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("echo".into(), Arc::new(ActEcho{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("ls".into(), Arc::new(ActLs{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("cp".into(), Arc::new(ActCp{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("num".into(), Arc::new(ActNum{}) as Arc<dyn Action + Send + Sync>);

        let m_buf = VecBuffer::new();
        let mut c = Commander {
            ctx,
            cmds,
            valid,
            confirmed: false,
            symbol_editor,
            cmd_editor: None,
            view_elements,
            out_port: port.outer()
                .to_sequence()
                .separate(make_label(" "))
                .to_grid_horizontal()
                .flatten(),

            msg_port: m_buf.get_port()
                .to_sequence()
                .flatten(),
            m_buf
        };

        c
    }
}

impl TerminalEditor for Commander {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.out_port.clone()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        if let (Some(cmd_editor), true) = (self.cmd_editor.as_ref(), self.confirmed) {
            match event {
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                    let mut c = cmd_editor.write().unwrap();
                    if c.nexd() == TreeNavResult::Exit {
                        // run
                        c.goto(TreeCursor::none());

                        TerminalEditorResult::Exit
                    } else {
                        TerminalEditorResult::Continue
                    }
                }
                event => {
                    cmd_editor.write().unwrap().handle_terminal_event(event)
                }
            }
        } else {
            match event {
                TerminalEvent::Input(Event::Key(Key::Char(' '))) |
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                    if let Some(editor) = &self.cmd_editor {
                        self.confirmed = true;
                        self.symbol_editor.up();
                        editor.write().unwrap().qpxev();

                        *self.view_elements.get_mut(1) = editor.read().unwrap().get_term_view();

                        self.m_buf.push(editor.read().unwrap().get_msg_port());
                        
                        if *event == TerminalEvent::Input(Event::Key(Key::Char('\n'))) {
                            return self.handle_terminal_event(event);
                        }
                    } else {
                        // undefined command
                        let mut b = VecBuffer::new();
                        b.push(nested::diagnostics::make_error(nested::terminal::make_label(&format!("invalid symbol {}", self.symbol_editor.get_string()))));
                        self.m_buf.clear();
                        self.m_buf.push(b.get_port().to_sequence());
                    }
                    
                    TerminalEditorResult::Continue
                }

                event => {
                    self.m_buf.clear();
                    let res = self.symbol_editor.handle_terminal_event(event);
                    let symbol = self.symbol_editor.get_string();

                    if let Some(action) = self.cmds.get(&symbol) {
                        let editor = action.make_editor(self.ctx.clone());

                        if self.view_elements.len() == 1 {
                            self.view_elements.push(editor.read().unwrap().get_term_view().map_item(|p,a| a.add_style_front(TerminalStyle::fg_color((80,80,80)))));
                        } else {
                            *self.view_elements.get_mut(1) = editor.read().unwrap().get_term_view().map_item(|p,a| a.add_style_front(TerminalStyle::fg_color((80,80,80))));
                        }

                        self.cmd_editor = Some(editor);
                        *self.valid.write().unwrap() = true;
                    } else {
                        /*
                        let mut b = VecBuffer::new();
                        b.push(nested::diagnostics::make_error(nested::terminal::make_label(&format!("invalid symbol {}", self.symbol_editor.get_string()))));
                        self.m_buf.push(b.get_port().to_sequence());
*/
                        self.cmd_editor = None;
                        *self.valid.write().unwrap() = false;

                        if self.view_elements.len() > 1 {
                            self.view_elements.remove(1);
                        }
                    }

                    res
                }
            }        
        }
    }
}

impl Diagnostics for Commander {
    fn get_msg_port(&self) -> OuterViewPort<dyn SequenceView<Item = nested::diagnostics::Message>> {
        self.msg_port.clone()
    }
}

impl TreeNav for Commander {
    fn get_cursor(&self) -> TreeCursor {
        if let (Some(cmd_editor), true) = (self.cmd_editor.as_ref(), self.confirmed) {
            cmd_editor.write().unwrap().get_cursor()
        } else {
            self.symbol_editor.get_cursor()
        }
    }
    fn get_cursor_warp(&self) -> TreeCursor {
        if let (Some(cmd_editor), true) = (self.cmd_editor.as_ref(), self.confirmed) {
            cmd_editor.write().unwrap().get_cursor_warp()
        } else {
            self.symbol_editor.get_cursor_warp()
        }
    }
    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        if let (Some(cmd_editor), true) = (self.cmd_editor.as_ref(), self.confirmed) {
            cmd_editor.write().unwrap().goby(dir)
        } else {
            self.symbol_editor.goby(dir)
        }
    }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        if let (Some(cmd_editor), true) = (self.cmd_editor.as_ref(), self.confirmed) {
            cmd_editor.write().unwrap().goto(cur)
        } else {
            self.symbol_editor.goto(cur)
        }
    }
}

impl Nested for Commander {}

