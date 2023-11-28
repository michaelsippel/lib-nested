use {
    cgmath::Vector2,
    nested::{
        edit_tree::NestedNode,
        repr_tree::{Context, ReprTree},
    },
    crate::{
        terminal::TermOutWriter, DisplaySegment, Terminal, TerminalAtom, TerminalCompositor,
        TerminalEvent, TerminalStyle, TerminalView,
    },
    r3vi::{
        buffer::singleton::*,
        view::{port::UpdateTask, singleton::*, ViewPort},
    },
    std::sync::{Arc, Mutex, RwLock},
    termion::event::{Event, Key},
};

pub struct TTYApplication {
    pub port: ViewPort<dyn TerminalView>,
    term_writer: Arc<TermOutWriter>,
}

impl TTYApplication {
    pub fn new(event_handler: impl Fn(TerminalEvent) + Send + Sync + 'static) -> Self {
        let port = ViewPort::new();
        let portmutex = Mutex::new(());
        let term = Terminal::new(port.outer());
        let term_writer = term.get_writer();

        async_std::task::spawn(TTYApplication::event_loop(term, port.clone(), Arc::new(event_handler)));
        async_std::task::spawn(TTYApplication::update_loop(port.clone()));

        TTYApplication {
            port,
            term_writer,
        }
    }

    /* this task handles all terminal events (e.g. key press, resize)
     */
    async fn event_loop(mut term: Terminal, port: ViewPort<dyn TerminalView>, event_handler: Arc<dyn Fn(TerminalEvent) + Send + Sync>) {
        loop {
            let ev = term.next_event().await;
            if ev == TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) {
                break;
            }

            event_handler( ev );
            port.update();
        }
    }

    /* this task will continuously pull forward
     * all notifications which are influencing
     * the view in `term_port`
     */
    async fn update_loop(port: ViewPort<dyn TerminalView>) {
        loop {
            port.update();
            async_std::task::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    /* write the changes in the view of `term_port` to the terminal
     */
    pub async fn show(&self) -> Result<(), std::io::Error> {
        self.term_writer.show().await
    }
}

