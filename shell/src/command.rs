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
        sequence::decorator::{Separate, Wrap, SeqDecorStyle},
        core::{TypeTerm, Context},
        core::{OuterViewPort, ViewPort},
        index::{IndexArea, IndexView},
        char_editor::CharEditor,
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView, make_label
        },
        tree_nav::{TreeCursor, TreeNav, TreeNavResult, TerminalTreeEditor},
        make_editor::make_editor,
        product::ProductEditor
    }
};

trait Action {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>;
}

pub struct ActCd {}
impl Action for ActCd {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>> {
        make_editor(
            ctx.clone(),
            &vec![ctx.read().unwrap().type_term_from_str("( Path )").unwrap()],
            1
        )
    }
}

pub struct ActEcho {}
impl Action for ActEcho {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>> {
        make_editor(
            ctx.clone(),
            &vec![ctx.read().unwrap().type_term_from_str("( String )").unwrap()],
            2
        )
    }
}

pub struct ActCp {}
impl Action for ActCp {
    fn make_editor(&self, ctx: Arc<RwLock<Context>>) -> Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>> {
        let depth = 1;
        Arc::new(RwLock::new(ProductEditor::new(depth, ctx.clone())
                             .with_t(Point2::new(0, 0), "Source ")
                             .with_n(Point2::new(1, 0), vec![ ctx.read().unwrap().type_term_from_str("( Path )").unwrap() ] )
                             .with_t(Point2::new(0, 1), "Destination ")
                             .with_n(Point2::new(1, 1), vec![ ctx.read().unwrap().type_term_from_str("( Path )").unwrap() ] )
                             .with_t(Point2::new(0, 2), "Options ")
                             .with_n(Point2::new(1, 2), vec![ ctx.read().unwrap().type_term_from_str("( List String )").unwrap() ] )
        )) as Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>
    }
}

pub struct Commander {
    ctx: Arc<RwLock<Context>>,
    cmds: HashMap<String, Arc<dyn Action + Send + Sync>>,

    symbol_editor: PTYListEditor<CharEditor>,
    cmd_editor: Option<Arc<RwLock<dyn TerminalTreeEditor + Send + Sync>>>,

    view_elements: VecBuffer<OuterViewPort<dyn TerminalView>>,
    out_port: OuterViewPort<dyn TerminalView>,
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
            0
        );

        view_elements.push(symbol_editor.get_term_view());

        let mut cmds = HashMap::new();
        cmds.insert("cd".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("echo".into(), Arc::new(ActEcho{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("ls".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("cp".into(), Arc::new(ActCp{}) as Arc<dyn Action + Send + Sync>);

        let mut c = Commander {
            ctx,
            cmds,
            symbol_editor,
            cmd_editor: None,
            view_elements,
            out_port: port.outer()
                .to_sequence()
                .separate(make_label(" "))
                .to_grid_horizontal()
                .flatten()
        };

        c
    }
}

impl TerminalEditor for Commander {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.out_port.clone()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            match event {
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                    // run
                    cmd_editor.write().unwrap().goto(TreeCursor::none());
                    TerminalEditorResult::Exit
                }
                event => {
                    cmd_editor.write().unwrap().handle_terminal_event(event)
                }
            }
        } else {
            match event {
                TerminalEvent::Input(Event::Key(Key::Char(' '))) |
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                    let symbol = self.symbol_editor.get_string();

                    if let Some(action) = self.cmds.get(&symbol) {
                        let editor = action.make_editor(self.ctx.clone());

                        self.symbol_editor.up();
                        self.view_elements.push(editor.read().unwrap().get_term_view());

                        editor.write().unwrap().qpxev();
                        self.cmd_editor = Some(editor);

                        if *event == TerminalEvent::Input(Event::Key(Key::Char('\n'))) {
                            return self.handle_terminal_event(event);
                        }
                    } else {
                        // undefined command
                    }

                    TerminalEditorResult::Continue
                }

                event => {
                    self.symbol_editor.handle_terminal_event(event)
                }
            }        
        }
    }
}

impl TreeNav for Commander {
    fn get_cursor(&self) -> TreeCursor {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.write().unwrap().get_cursor()
        } else {
            self.symbol_editor.get_cursor()
        }
    }
    fn get_cursor_warp(&self) -> TreeCursor {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.write().unwrap().get_cursor_warp()
        } else {
            self.symbol_editor.get_cursor_warp()
        }
    }
    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.write().unwrap().goby(dir)
        } else {
            self.symbol_editor.goby(dir)
        }
    }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.write().unwrap().goto(cur)
        } else {
            self.symbol_editor.goto(cur)
        }
    }

}

impl TerminalTreeEditor for Commander {}

