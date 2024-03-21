//! This example shows how to:
//!  - initialize the TTY backend (`lib-nestetd-tty`),
//!  - create a simple 'Hello World' output,
//!  - create color gradients on the outputted text
//!    utilizing basic projection functionality from `lib-r3vi`,
//!  - perform basic layouting & compositing.

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
    /* Initialize our terminal.
     */
    let tty_app = TTYApplication::new(|event| { /* handle event */ });

    /* Setup our "root" view of the application.
     * This will be the compositor, which is able to
     * mix multiple `TerminalView`-Views together.
     * Its output is routed to the `app.port` Viewport,
     * so it will be displayed on TTY-output.
     */
    let compositor = TerminalCompositor::new(tty_app.port.inner());

    /* Add the label 'test' at position (7, 2)
     */
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label("test").offset(Vector2::new(7, 2)));

    /* Add a 'Hello World' label at position (5, 3)
     * and set a coloring determined by formula from
     * the position of each character.
     */
    compositor.write().unwrap().push(
        nested_tty::make_label("Hello World")
            .map_item(|p, a| {
                a.add_style_back(TerminalStyle::fg_color(((25 * p.x % 255) as u8, 200, 0)))
            })
            .offset(Vector2::new(5, 3)),
    );

    /* write the changes in the root-view to the terminal
     */
    tty_app.show().await.expect("output error!");
}

