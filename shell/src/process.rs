use {
    std::{
        sync::Arc,
        process::Command,
        os::unix::io::{FromRawFd, AsRawFd},
    },
    std::sync::RwLock,
    termion::event::{Key, Event},
    cgmath::Point2,
    nested::{
        core::{ViewPort, OuterViewPort, InnerViewPort, Observer},
        singleton::{SingletonView, SingletonBuffer},
        sequence::{SequenceView, SequenceViewExt},
        index::buffer::IndexBuffer,
        vec::VecBuffer,
        terminal::{TerminalAtom, TerminalStyle, TerminalView, TerminalEvent, TerminalEditor, TerminalEditorResult, make_label},
        tree_nav::{TreeNav, TreeNavResult, TerminalTreeEditor, TreeCursor},
        list::{ListCursorMode, ListEditor, ListEditorStyle, sexpr::ListDecoration},
        string_editor::CharEditor,
    },
    crate::pty::{PTY, PTYStatus}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ProcessArg {
    editor: ListEditor< CharEditor,
                        Box<dyn Fn() -> Arc<RwLock<CharEditor>> + Send + Sync + 'static> >
}

impl ProcessArg {
    pub fn get_data_port(&self) -> OuterViewPort<dyn SequenceView<Item = char>> {
        self.editor.get_data_port()
            .map(
                |char_editor| char_editor.read().unwrap().get_data_port().get_view().unwrap().get().unwrap()
            )
    }
}

impl TerminalEditor for ProcessArg {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor
            .get_seg_seq_view()
            .to_grid_horizontal()
            .flatten()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(' '))) |
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.editor.up();
                TerminalEditorResult::Exit
            }

            event => self.editor.handle_terminal_event(event)
        }
    }
}

impl TreeNav for ProcessArg {
    fn get_cursor(&self) -> TreeCursor { self.editor.get_cursor() }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult  { self.editor.goto(cur) }
    fn goto_home(&mut self) -> TreeNavResult  { self.editor.goto_home() }
    fn goto_end(&mut self) -> TreeNavResult  { self.editor.goto_end() }
    fn pxev(&mut self) -> TreeNavResult  { self.editor.pxev() }
    fn nexd(&mut self) -> TreeNavResult  { self.editor.nexd() }
    fn up(&mut self) -> TreeNavResult { self.editor.up() }
    fn dn(&mut self) -> TreeNavResult { self.editor.dn() }
}

pub struct ProcessLauncher {
    cmd_editor: ListEditor<
            ProcessArg, Box<dyn Fn() -> Arc<RwLock<ProcessArg>> + Send + Sync + 'static>
        >,
    pty: Option<crate::pty::PTY>,
    ptybox: Arc<RwLock<crate::ascii_box::AsciiBox>>,
    suspended: bool,

    pty_port: ViewPort<dyn TerminalView>,
    status_port: ViewPort<dyn SingletonView<Item = PTYStatus>>,

    comp_port: ViewPort<dyn TerminalView>,
    compositor: Arc<RwLock<nested::terminal::TerminalCompositor>>
}

impl ProcessLauncher {
    pub fn new() -> Self {
        let pty_port = ViewPort::new();
        let status_port = ViewPort::new();
        let comp_port = ViewPort::new();
        let box_port = ViewPort::<dyn TerminalView>::new();
        let compositor = nested::terminal::TerminalCompositor::new(comp_port.inner());

        let cmd_editor = ListEditor::new(
                Box::new(
                    || {
                        Arc::new(RwLock::new(ProcessArg {
                        editor: ListEditor::new(
                            Box::new(
                                || {
                                    Arc::new(RwLock::new(CharEditor::new()))
                                }
                            ),
                            ListEditorStyle::Plain)
                        }))
                    }
                ) as Box::<dyn Fn() -> Arc<RwLock<ProcessArg>> + Send + Sync>,
                ListEditorStyle::Plain
        );

        compositor.write().unwrap().push(
            box_port.outer()
                .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((90, 120, 100))))
        );
        compositor.write().unwrap().push(
            cmd_editor
                .get_seg_seq_view()
                .decorate("$(", ")", " ", 0)
                .to_grid_horizontal()
                .flatten()
        );

        ProcessLauncher {
            cmd_editor,
            pty: None,
            ptybox: crate::ascii_box::AsciiBox::new(
                    cgmath::Vector2::new(80, 25),
                    pty_port.outer()
                        .map_item(|_,a:&TerminalAtom| a.add_style_back(TerminalStyle::fg_color((230, 230, 230)))),
                    box_port.inner()
            ),
            suspended: false,
            pty_port,
            status_port,
            comp_port,
            compositor
        }
    }

    pub fn launch_pty(&mut self) {
        let mut strings = Vec::new();

        let v = self.cmd_editor.get_data_port().get_view().unwrap();
        for i in 0 .. v.len().unwrap_or(0) {
            let arg_view = v.get(&i).unwrap().read().unwrap().get_data_port().get_view().unwrap();
            strings.push(arg_view.iter().collect::<String>());
        }

        if strings.len() > 0 {
            // Spawn a shell into the pty
            let mut cmd = crate::pty::CommandBuilder::new(strings[0].as_str());
            cmd.args(&strings[1..]);

            self.cmd_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![]
            });
            
            self.pty = PTY::new(cmd, cgmath::Vector2::new(120, 40), self.pty_port.inner(), self.status_port.inner());
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
        if let PTYStatus::Done{ status } = self.status_port.outer().get_view().get() {
            self.pty = None;
            self.suspended = false;
        }

        match event {
            TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => {
                // todo: sigterm instead of kill?
                if let Some(mut pty) = self.pty.as_mut() {
                    pty.kill();
                }

                self.pty = None;
                self.suspended = false;
                self.cmd_editor.goto(TreeCursor {
                    leaf_mode: ListCursorMode::Insert,
                    tree_addr: vec![]
                });                
                TerminalEditorResult::Exit
            },
            TerminalEvent::Input(Event::Key(Key::Ctrl('z'))) => {
                self.suspended = true;
                self.cmd_editor.goto(TreeCursor {
                    leaf_mode: ListCursorMode::Insert,
                    tree_addr: vec![]
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
                        event => self.cmd_editor.handle_terminal_event(event)                    
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

    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult {
        self.suspended = false;
        if let PTYStatus::Done{status} = self.status_port.outer().get_view().get() {
            self.pty = None;
        }

        if self.pty.is_none() {
            self.cmd_editor.goto(cur)
        } else {
            self.cmd_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Select,
                tree_addr: vec![]
            });
            TreeNavResult::Continue
        }
    }

    fn goto_home(&mut self) -> TreeNavResult {
        self.cmd_editor.goto_home()
    }

    fn goto_end(&mut self) -> TreeNavResult {
        self.cmd_editor.goto_end()
    }

    fn pxev(&mut self) -> TreeNavResult {
        self.cmd_editor.pxev()
    }

    fn nexd(&mut self) -> TreeNavResult {
        self.cmd_editor.nexd()
    }

    fn up(&mut self) -> TreeNavResult {
        self.cmd_editor.up()
    }

    fn dn(&mut self) -> TreeNavResult {
        self.cmd_editor.dn()
    }
}

