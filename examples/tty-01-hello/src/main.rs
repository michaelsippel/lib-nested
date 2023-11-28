extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::repr_tree::Context,
    nested_tty::{Terminal, TerminalCompositor, TTYApplication, TerminalEvent, TerminalStyle, TerminalView},
    r3vi::view::{port::UpdateTask, ViewPort},
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

#[async_std::main]
async fn main() {
    /* initialize our terminal
     */
    let tty_app = TTYApplication::new(|event| { /* handle event */ });

    /* populate the view in `term_port`
     */
    let compositor = TerminalCompositor::new(tty_app.port.inner());

    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label("test").offset(Vector2::new(7, 2)));

    compositor.write().unwrap().push(
        nested_tty::make_label("Hello World")
            .map_item(|p, a| {
                a.add_style_back(TerminalStyle::fg_color(((25 * p.x % 255) as u8, 200, 0)))
            })
            .offset(Vector2::new(5, 3)),
    );

    /* write the changes in the view of `term_port` to the terminal
     */
    tty_app.show().await.expect("output error!");
}

