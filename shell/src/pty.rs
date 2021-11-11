
use {
    termion::event::{Key, Event},
    std::sync::Mutex,
    nested::{
        core::{InnerViewPort},
        singleton::{SingletonView, SingletonBuffer},
        terminal::{TerminalView, TerminalEvent, TerminalEditorResult}
    }
};

pub use portable_pty::CommandBuilder;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTY {
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>
}

impl PTY {
    pub fn new(
        cmd: portable_pty::CommandBuilder,
        term_port: InnerViewPort<dyn TerminalView>,
        status_port: InnerViewPort<dyn SingletonView<Item = Option<portable_pty::ExitStatus>>>
    ) -> Option<Self> {

        // Create a new pty
        let mut pair = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
            rows: 25,
            cols: 120,

            // Not all systems support pixel_width, pixel_height,
            // but it is good practice to set it to something
            // that matches the size of the selected font.  That
            // is more complex than can be shown here in this
            // brief example though!
            pixel_width: 0,
            pixel_height: 0,
        }).unwrap();

        if let Ok(mut child) = pair.slave.spawn_command(cmd) {
            let mut reader = pair.master.try_clone_reader().unwrap();

            async_std::task::spawn_blocking(
                move || {
                    nested::terminal::ansi_parser::read_ansi_from(&mut reader, term_port);
                });

            async_std::task::spawn_blocking(
                move || {
                    let mut status_buf = SingletonBuffer::new(None, status_port);
                    if let Ok(status) = child.wait() {
                        status_buf.set(Some(status));
                    }
                }
            );

            Some(PTY {
                master: Mutex::new(pair.master)
            })
        } else {
            None
        }
    }

    pub fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                self.master.lock().unwrap().write(&[13]).unwrap();
                TerminalEditorResult::Continue
            },
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                write!(self.master.lock().unwrap(), "{}", c);
                TerminalEditorResult::Continue
            },
            TerminalEvent::Input(Event::Key(Key::Esc)) => {
                self.master.lock().unwrap().write(&[0x1b]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Backspace)) => {
                self.master.lock().unwrap().write(&[0x8]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::F(n))) => {
                self.master.lock().unwrap().write(&[
                    0x1b,
                    0x0a,
                    match n {
                        11 => 133,
                        12 => 134,
                        n => 58 + n
                    }
                ]).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Up)) => {
                self.master.lock().unwrap().write(&[0, b'\x1B', b'[', b'A']).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Down)) => {
                self.master.lock().unwrap().write(&[0, b'\x1B', b'[', b'B']).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Right)) => {
                self.master.lock().unwrap().write(&[0, b'\x1B', b'[', b'C']).unwrap();
                TerminalEditorResult::Continue
            }
            TerminalEvent::Input(Event::Key(Key::Left)) => {
                self.master.lock().unwrap().write(&[0, b'\x1B', b'[', b'D']).unwrap();
                TerminalEditorResult::Continue
            }
            _ => {
                TerminalEditorResult::Exit
            }
        }
    }
}

