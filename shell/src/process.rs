use {
    std::{
        sync::Arc,
        process::Command,
        os::unix::io::{FromRawFd, AsRawFd},
    },
    std::sync::RwLock,
    tty::{FileDesc, TtyServer},
    termion::event::{Key, Event},
    cgmath::Point2,
    nested::{
        core::{ViewPort, OuterViewPort, Observer},
        singleton::{SingletonView, SingletonBuffer},
        sequence::{SequenceView, SequenceViewExt},
        index::buffer::IndexBuffer,
        vec::VecBuffer,
        terminal::{TerminalAtom, TerminalStyle, TerminalView, TerminalEvent, TerminalEditor, TerminalEditorResult, make_label},
        tree_nav::{TreeNav, TreeNavResult, TerminalTreeEditor, TreeCursor},
        list::{ListEditor, ListEditorStyle, sexpr::ListDecoration},
        string_editor::CharEditor,
    }
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
    editor: ListEditor< ProcessArg,
                        Box<dyn Fn() -> Arc<RwLock<ProcessArg>> + Send + Sync + 'static> >
}

impl ProcessLauncher {
    pub fn new() -> Self {
        ProcessLauncher {
            editor: ListEditor::new(
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
                ),
                ListEditorStyle::Plain
            )
        }
    }

    pub fn launch(&mut self) -> OuterViewPort<dyn TerminalView> {
        self.up();
        self.up();
        
        let mut strings = Vec::new();

        let v = self.editor.get_data_port().get_view().unwrap();
        for i in 0 .. v.len().unwrap_or(0) {
            let arg_view = v.get(&i).unwrap().read().unwrap().get_data_port().get_view().unwrap();
            strings.push(arg_view.iter().collect::<String>());
        }

        if strings.len() > 0 {
            let stdin = FileDesc::new(libc::STDIN_FILENO, false);
            let mut server = match TtyServer::new(Some(&stdin)) {
                Ok(s) => s,
                Err(e) => { return make_label(&format!("Error TTY server: {}", e)); },
            };

            let mut cmd = std::process::Command::new(strings[0].as_str());
            cmd.args(&strings[1..]).stdin(std::process::Stdio::null());

            let process = match server.spawn(cmd) {
                Ok(p) => p,
                Err(e) => { return make_label(&format!("Failed to execute process: {}", e));},
            };

            if let Ok(mut term_view_proc) = std::process::Command::new("./target/release/ansi_parser")
                .stdin(unsafe{ std::process::Stdio::from_raw_fd(server.get_master().as_raw_fd()) })
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                let mut term_view_bin = term_view_proc.stdout.unwrap();
                /*
                //let mut term_view_bin = async_std::io::BufReader::new(unsafe { async_std::fs::File::from_raw_fd( term_view_proc.stdout.unwrap().as_raw_fd() ) } );
                //let mut tv_stream = async_bincode::AsyncBincodeReader::<_, (Point2<i16>, Option<TerminalAtom>)>::from(term_view_bin);

                async_std::task::spawn(
                    async move {
                 */

                let output_view_port = ViewPort::new();
                let mut output_buf = IndexBuffer::new(output_view_port.inner());

                while let Ok((pos, atom)) = bincode::deserialize_from(&mut term_view_bin) {
                    if let Some(a) = atom {
                        output_buf.insert(pos, a);
                    } else {
                        output_buf.remove(pos);
                    }
                }
//                    });
                output_view_port.outer()
            } else {
                make_label("Failed to spawn ansi parser process")
                    .map_item(|idx, a| a.add_style_back(TerminalStyle::fg_color((200,0,0))))
            }
        } else {
            make_label("no command")
                .map_item(|idx, a| a.add_style_back(TerminalStyle::fg_color((200,0,0))))
        }
    }
}

impl TerminalEditor for ProcessLauncher {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.editor
            .get_seg_seq_view()
            .decorate("$(", ")", " ", 0)
            .to_grid_horizontal()
            .flatten()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                // launch command
                self.editor.up();
                self.editor.up();
                TerminalEditorResult::Exit
            }

            event => self.editor.handle_terminal_event(event)
        }
    }    
}

impl TreeNav for ProcessLauncher {
    fn get_cursor(&self) -> TreeCursor { self.editor.get_cursor() }
    fn goto(&mut self, cur: TreeCursor) -> TreeNavResult  { self.editor.goto(cur) }
    fn goto_home(&mut self) -> TreeNavResult  { self.editor.goto_home() }
    fn goto_end(&mut self) -> TreeNavResult  { self.editor.goto_end() }
    fn pxev(&mut self) -> TreeNavResult  { self.editor.pxev() }
    fn nexd(&mut self) -> TreeNavResult  { self.editor.nexd() }
    fn up(&mut self) -> TreeNavResult { self.editor.up() }
    fn dn(&mut self) -> TreeNavResult { self.editor.dn() }
}

