use {
    crate::pty::{PTYStatus, PTY},
    nested::{
        core::{OuterViewPort, ViewPort},
        list::{ListCursorMode, PTYListEditor},
        sequence::{SequenceView, SequenceViewExt, decorator::{SeqDecorStyle, Separate, Wrap}},
        singleton::SingletonView,
        char_editor::CharEditor,
        terminal::{
            TerminalAtom, TerminalEditor, TerminalEditorResult, TerminalEvent, TerminalStyle,
            TerminalView,
        },
        tree::{TreeCursor, TreeNav, TreeNavResult},
        diagnostics::Diagnostics,
        Nested
    },
    std::sync::Arc,
    std::sync::RwLock,
    termion::event::{Event, Key},
    cgmath::Vector2
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProcessArg {
    editor:
        PTYListEditor<CharEditor>,
}

impl ProcessArg {
    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = char>> {
        self.editor.get_data_port().map(|char_editor| {
            char_editor
                .read()
                .unwrap()
                .get_port()
                .get_view()
                .unwrap()
                .get()
                .unwrap()
        })
    }
}

impl TerminalEditor for ProcessArg {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor.get_term_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(' ')))
            | TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.editor.up();
                TerminalEditorResult::Exit
            }

            event => self.editor.handle_terminal_event(event),
        }
    }
}

impl TreeNav for ProcessArg {
    fn get_cursor(&self) -> TreeCursor {
        self.editor.get_cursor()
    }
    fn get_cursor_warp(&self) -> TreeCursor {
        self.editor.get_cursor_warp()
    }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        self.editor.goto(cur)
    }
    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        self.editor.goby(dir)
    }
}

impl Diagnostics for ProcessArg {    
}

impl Nested for ProcessArg {}

pub struct ProcessLauncher {
    cmd_editor: PTYListEditor<ProcessArg>,
    pty: Option<crate::pty::PTY>,
    _ptybox: Arc<RwLock<crate::ascii_box::AsciiBox>>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,
    status_port: ViewPort<dyn SingletonView<Item = PTYStatus>>,

    comp_port: ViewPort<dyn TerminalView>,
    _compositor: Arc<RwLock<nested::terminal::TerminalCompositor>>,
}

impl ProcessLauncher {
    pub fn new() -> Self {
        let pty_port = ViewPort::new();
        let status_port = ViewPort::new();
        let comp_port = ViewPort::new();
        let box_port = ViewPort::<dyn TerminalView>::new();
        let compositor = nested::terminal::TerminalCompositor::new(comp_port.inner());

        let cmd_editor = PTYListEditor::new(
            Box::new(|| {
                Arc::new(RwLock::new(ProcessArg {
                    editor: PTYListEditor::new(
                        Box::new(|| Arc::new(RwLock::new(CharEditor::new()))),
                        SeqDecorStyle::Plain,
                        '\n',
                        1
                    ),
                }))
            }) as Box<dyn Fn() -> Arc<RwLock<ProcessArg>> + Send + Sync>,
            SeqDecorStyle::HorizontalSexpr,
            ' ',
            0
        );

        compositor.write().unwrap().push(
            box_port
                .outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((90, 120, 100)))),
        );
        compositor.write().unwrap().push(
            cmd_editor.get_term_view()
        );

        ProcessLauncher {
            cmd_editor,
            pty: None,
            _ptybox: crate::ascii_box::AsciiBox::new(
                cgmath::Vector2::new(0, 0),
                pty_port.outer().map_item(|_, a: &TerminalAtom| {
                    a.add_style_back(TerminalStyle::fg_color((230, 230, 230)))
                }),
                box_port.inner(),
            ),
            suspended: false,
            pty_port,
            status_port,
            comp_port,
            _compositor: compositor,
        }
    }

    pub fn launch_pty(&mut self) {
        let mut strings = Vec::new();

        let v = self.cmd_editor.get_data_port().get_view().unwrap();
        for i in 0..v.len().unwrap_or(0) {
            let arg_view = v
                .get(&i)
                .unwrap()
                .read()
                .unwrap()
                .get_data_port()
                .get_view()
                .unwrap();
            strings.push(arg_view.iter().collect::<String>());
        }

        if strings.len() > 0 {
            // Spawn a shell into the pty
            let mut cmd = crate::pty::CommandBuilder::new(strings[0].as_str());
            cmd.args(&strings[1..]);
            cmd.cwd(".");

            self.cmd_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![],
            });

            self.pty = PTY::new(
                cmd,
                cgmath::Vector2::new(120, 40),
                self.pty_port.inner(),
                self.status_port.inner(),
            );
        }
    }

    pub fn is_captured(&self) -> bool {
        self.pty.is_some() && !self.suspended
    }
}

impl TerminalEditor for ProcessLauncher {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.comp_port.outer()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        // todo: move to observer of status view
        if let PTYStatus::Done { status: _ } = self.status_port.outer().get_view().get() {
            self.pty = None;
            self.suspended = false;
        }

        match event {
            TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => {
                // todo: sigterm instead of kill?
                if let Some(pty) = self.pty.as_mut() {
                    pty.kill();
                }

                self.pty = None;
                self.suspended = false;
                self.cmd_editor.goto(TreeCursor {
                    leaf_mode: ListCursorMode::Insert,
                    tree_addr: vec![],
                });
                TerminalEditorResult::Exit
            }
            TerminalEvent::Input(Event::Key(Key::Ctrl('z'))) => {
                self.suspended = true;
                self.cmd_editor.goto(TreeCursor {
                    leaf_mode: ListCursorMode::Insert,
                    tree_addr: vec![],
                });
                TerminalEditorResult::Exit
            }
            event => {
                if let Some(pty) = self.pty.as_mut() {
                    pty.handle_terminal_event(event);
                    TerminalEditorResult::Continue
                } else {
                    match event {
                        TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                            // launch command
                            self.launch_pty();
                            TerminalEditorResult::Continue
                        }
                        event => self.cmd_editor.handle_terminal_event(event),
                    }
                }
            }
        }
    }
}

impl TreeNav for ProcessLauncher {
    fn get_cursor(&self) -> TreeCursor {
        self.cmd_editor.get_cursor()
    }
    fn get_cursor_warp(&self) -> TreeCursor {
        self.cmd_editor.get_cursor_warp()
    }

    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        self.suspended = false;
        if let PTYStatus::Done { status: _ } = self.status_port.outer().get_view().get() {
            self.pty = None;
        }

        if self.pty.is_none() {
            self.cmd_editor.goto(cur)
        } else {
            self.cmd_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Select,
                tree_addr: vec![],
            });
            TreeNavResult::Continue
        }
    }

    fn goby(&mut self, dir: Vector2<isize>) -> TreeNavResult {
        self.cmd_editor.goby(dir)
    }

}

impl Diagnostics for ProcessLauncher {
}

impl Nested for ProcessLauncher {}

