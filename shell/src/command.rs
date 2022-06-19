use {
    std::{
        sync::{Arc, RwLock},
        collections::HashMap
    },
    cgmath::{Point2},
    termion::event::{Event, Key},
    nested::{
        list::{sexpr::ListDecoration, ListEditor, ListEditorStyle},
        core::TypeTerm,
        core::{OuterViewPort, ViewPort},
        index::{IndexArea, IndexView},
        string_editor::StringEditor,
        vec::VecBuffer,
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView, make_label
        },
        tree_nav::{TreeCursor, TreeNav, TreeNavResult},
    }
};

trait Action {
    fn make_editor(&self) ->
        (Arc<RwLock<dyn TerminalEditor + Send + Sync>>,
         Arc<RwLock<dyn TreeNav + Send + Sync>>);
}

pub struct ActCd {}
impl Action for ActCd {
    fn make_editor(&self) ->
        (Arc<RwLock<dyn TerminalEditor + Send + Sync>>,
         Arc<RwLock<dyn TreeNav + Send + Sync>>)
    {
        let ed =
            Arc::new(RwLock::new(ListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(StringEditor::new()))
                }) as Box<dyn Fn() -> Arc<RwLock<StringEditor>> + Send + Sync>,
                ListEditorStyle::HorizontalSexpr,
            )));
        //Arc::new(RwLock::new(StringEditor::new()));

        (ed.clone() as Arc<RwLock<dyn TerminalEditor + Send + Sync>>, ed as Arc<RwLock<dyn TreeNav + Send + Sync>>)
    }
}


pub struct Commander {
    cmds: HashMap<String, Arc<dyn Action + Send + Sync>>,

    symbol_editor: StringEditor,
    cmd_editor: Option<(
        Arc<RwLock<dyn TerminalEditor + Send + Sync>>,
        Arc<RwLock<dyn TreeNav + Send + Sync>>
    )>,

    view_elements: VecBuffer<OuterViewPort<dyn TerminalView>>,
    out_port: OuterViewPort<dyn TerminalView>,
}

impl Commander {
    pub fn new() -> Self {
        let port = ViewPort::new();
        let mut view_elements = VecBuffer::new(port.inner());
        let symbol_editor = StringEditor::new();

        view_elements.push(symbol_editor.get_plain_editor_view());

        let mut cmds = HashMap::new();
        cmds.insert("cd".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("echo".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);
        cmds.insert("ls".into(), Arc::new(ActCd{}) as Arc<dyn Action + Send + Sync>);

        let mut c = Commander {
            cmds,
            symbol_editor,
            cmd_editor: None,
            view_elements,
            out_port: port.outer()
                .to_sequence()
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
                    cmd_editor.1.write().unwrap().up();
                    TerminalEditorResult::Exit
                }
                event => {
                    cmd_editor.0.write().unwrap().handle_terminal_event(event)
                }
            }
        } else {
            match event {
                TerminalEvent::Input(Event::Key(Key::Char(' '))) |
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                    let symbol = self.symbol_editor.get_string();

                    if let Some(action) = self.cmds.get(&symbol) {
                        let editor = action.make_editor();

                        self.symbol_editor.up();
                        self.view_elements.push(make_label(" "));
                        self.view_elements.push(editor.0.read().unwrap().get_term_view());

                        editor.1.write().unwrap().goto_home();
                        self.cmd_editor = Some(editor);
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
            cmd_editor.1.write().unwrap().get_cursor()
        } else {
            self.symbol_editor.get_cursor()
        }
    }

    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().goto(cur)
        } else {
            self.symbol_editor.goto(cur)
        }
    }
    fn goto_home(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().goto_home()
        } else {
            self.symbol_editor.goto_home()
        }
    }
    fn goto_end(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().goto_end()
        } else {
            self.symbol_editor.goto_end()
        }
    }
    fn pxev(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().pxev()
        } else {
            self.symbol_editor.pxev()
        }
    }
    fn nexd(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().nexd()
        } else {
            self.symbol_editor.nexd()
        }
    }
    fn up(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().up()
        } else {
            self.symbol_editor.up()
        }
    }
    fn dn(&mut self) -> TreeNavResult {
        if let Some(cmd_editor) = self.cmd_editor.as_ref() {
            cmd_editor.1.write().unwrap().dn()
        } else {
            self.symbol_editor.dn()
        }
    }
}


