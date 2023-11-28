extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::{
        editTree::NestedNode,
        reprTree::{Context, ReprTree},
    },
    nested_tty::{
        terminal::TermOutWriter, DisplaySegment, Terminal, TerminalAtom, TerminalCompositor,
        TerminalEvent, TerminalStyle, TerminalView,
        TTYApplication
    },
    r3vi::{
        buffer::singleton::*,
        view::{port::UpdateTask, singleton::*, ViewPort},
    },
    std::sync::{Arc, Mutex, RwLock},
    termion::event::{Event, Key},
};

#[async_std::main]
async fn main() {
    let app = TTYApplication::new( |ev| { /* event handler */ } );
    let compositor = TerminalCompositor::new(app.port.inner());

    /* setup context & create Editor-Tree
     */
    let ctx = Arc::new(RwLock::new(Context::default()));

    // abstract data
    let rt = ReprTree::from_char(&ctx, 'λ');

    let mut node = Context::make_node(
        &ctx,
        // node type
        Context::parse(&ctx, "Char"),
        // depth
        SingletonBuffer::new(0).get_port(),
    )
    .unwrap();

    /* add a display view to the node
     */
    let char_view = rt
        .read()
        .unwrap()
        .get_port::<dyn SingletonView<Item = char>>()
        .expect("unable to get Char-view")
        .map(move |c| TerminalAtom::from(if c == '\0' { ' ' } else { c }))
        .to_grid();

    let mut display_rt = ReprTree::new(Context::parse(&ctx, "Display"));

    display_rt.insert_branch(ReprTree::new_leaf(
        Context::parse(&ctx, "TerminalView"),
        char_view.into(),
    ));

    node = node.set_view(Arc::new(RwLock::new(display_rt)));

    compositor.write().unwrap().push(
        nested_tty::make_label("Hello World")
            .map_item(|p, a| {
                a.add_style_back(TerminalStyle::fg_color(((25 * p.x % 255) as u8, 200, 0)))
            })
            .offset(Vector2::new(5, 0)),
    );

    compositor.write().unwrap()
        .push(nested_tty::make_label("Char").offset(Vector2::new(0, 2)));

    compositor.write().unwrap()
        .push(node.display_view().offset(Vector2::new(15, 2)));

    compositor.write().unwrap()
        .push(nested_tty::make_label("<List Char>").offset(Vector2::new(0, 3)));

    compositor.write().unwrap()
        .push(nested_tty::make_label("---").offset(Vector2::new(15, 3)));

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
