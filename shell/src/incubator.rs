
use {
    nested::commander::Commander,
    nested::terminal::TerminalEvent
};

struct Incubator {
    
}

impl Commander for Incubator {
    type Cmd = TerminalEvent;

    fn send_cmd(&mut self, cmd: &TerminalEvent) {
        
    }
}

