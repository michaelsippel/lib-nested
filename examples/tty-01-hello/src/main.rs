extern crate r3vi;
extern crate nested;
extern crate nested_tty;
extern crate termion;
extern crate cgmath;

use {
    r3vi::view::{
        ViewPort,
        port::UpdateTask
    },
    nested::{
        tree::{NestedNode},
        type_system::{Context, ReprTree}
    },
    nested_tty::{
        Terminal, TerminalView, TerminalEvent,
        TerminalStyle,
        TerminalCompositor
    },
    cgmath::Vector2,
    termion::event::{Event, Key},
    std::sync::{Arc, RwLock}
};


/* this task handles all terminal events (e.g. key press, resize)
 */
pub async fn event_loop(
    mut term: Terminal,
    term_port: ViewPort<dyn TerminalView>,
    portmutex: Arc<RwLock<()>>
) {
    loop {
        let ev = term.next_event().await;
        let _l = portmutex.write().unwrap();

        if ev == TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) {
            break;
        }
        term_port.update();
    }
}

/* this task will continuously pull forward
 * all notifications which are influencing
 * the view in `term_port`
 */
pub async fn update_loop(
    term_port: ViewPort<dyn TerminalView>,
    portmutex: Arc<RwLock<()>>
) {
    loop {
        {
            let _l = portmutex.write().unwrap();
            term_port.update();
        }
        async_std::task::sleep(std::time::Duration::from_millis(500)).await;
    }
}

#[async_std::main]
async fn main() {
    /* initialize our terminal
     */
    let term_port = ViewPort::new();
  
    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    let portmutex = Arc::new(RwLock::new(()));

    /* spawn event-handling & updating tasks
     */
    async_std::task::spawn(
        update_loop(term_port.clone(), portmutex.clone()));

    async_std::task::spawn(
        event_loop(term, term_port.clone(), portmutex.clone()));

    /* populate the view in `term_port`
     */
    let compositor = TerminalCompositor::new(term_port.inner());

    compositor.write().unwrap().push(
        nested_tty::make_label("test")
            .offset(Vector2::new(7,2)));

    compositor.write().unwrap().push(
        nested_tty::make_label("Hello World")
            .map_item(|p,a|
                a.add_style_back(
                    TerminalStyle::fg_color(( (25*p.x%255) as u8, 200, 0 )) ))
            .offset(Vector2::new(5, 3)));

    /* write the changes in the view of `term_port` to the terminal
     */
    term_writer.show().await.expect("output error!");
}


