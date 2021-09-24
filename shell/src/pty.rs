
use {
    termion::event::{Key, Event},
    std::sync::Mutex,
    nested::{
        core::{InnerViewPort},
        terminal::{TerminalView, TerminalEvent}
    }
};

pub use portable_pty::CommandBuilder;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct PTY {
    master: Mutex<Box<dyn portable_pty::MasterPty + Send>>,
    child: Box<dyn portable_pty::Child + Send + Sync>
}

impl PTY {
    pub fn new(
        cmd: portable_pty::CommandBuilder,
        port: InnerViewPort<dyn TerminalView>
    ) -> Option<Self> {

        // Create a new pty
        let mut pair = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
            rows: 25,
            cols: 80,

            // Not all systems support pixel_width, pixel_height,
            // but it is good practice to set it to something
            // that matches the size of the selected font.  That
            // is more complex than can be shown here in this
            // brief example though!
            pixel_width: 0,
            pixel_height: 0,
        }).unwrap();

        if let Ok(child) = pair.slave.spawn_command(cmd) {
            let mut reader = pair.master.try_clone_reader().unwrap();

            async_std::task::spawn_blocking(
                move || {
                    nested::terminal::ansi_parser::read_ansi_from(&mut reader, port);
                });
            
            Some(PTY {
                master: Mutex::new(pair.master),
                child
            })
        } else {
            None
        }
    }

    pub fn get_status(&mut self) -> bool {
        if let Ok(Some(status)) = self.child.try_wait() {
            true
        } else {
            false
        }
    }

    pub fn handle_terminal_event(&mut self, event: &TerminalEvent) {
        match event {
            TerminalEvent::Input(Event::Key(Key::Char(c))) => {
                write!(self.master.lock().unwrap(), "{}", c);
            }
            _ => {
            }
        }
    }
}

