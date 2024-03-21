//! Similarly to `tty-02-digit`, a editor is created
//! but of type <List Char>.
//! The contents of the editor can be retrieved by
//! a morphism from the `EditTree` node.
//! To demonstrate that, the values are are mapped
//! to the TTY-display in different form.

extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::{
        editors::ObjCommander,
        repr_tree::{Context, ReprTree, ReprTreeExt},
        edit_tree::{EditTree}
    },
    nested_tty::{
        DisplaySegment, TTYApplication,
        TerminalCompositor, TerminalStyle, TerminalView,
        TerminalAtom, TerminalEvent
    },
    r3vi::{
        buffer::{singleton::*, vec::*},
        view::{port::UpdateTask, list::*}
    },
    std::sync::{Arc, RwLock},
};

#[async_std::main]
async fn main() {
    /* setup context
     */
    let ctx = Arc::new(RwLock::new(Context::new()));
    nested::editors::char::init_ctx( ctx.clone() );
    nested::editors::digit::init_ctx( ctx.clone() );
    nested::editors::integer::init_ctx( ctx.clone() );
    nested::editors::list::init_ctx( ctx.clone() );
    nested_tty::setup_edittree_hook(&ctx);


    /* Create a Representation-Tree of type <List Char>
     */
    let rt_string = ReprTree::new_arc( Context::parse(&ctx, "<List Char>") );

    /* Setup an Editor for this ReprTree
     * (by adding the representation <List Char>~EditTree to the ReprTree)
     */
    let edittree_list = ctx.read().unwrap()
        .setup_edittree(
            rt_string.clone(),
            SingletonBuffer::new(0).get_port());

    /* In order to get acces to the values that are modified by the Editor,
     * we apply a morphism that, given the List of Edit-Trees, extracts
     * the value from each EditTree and shows them in a ListView.
     */
    ctx.read().unwrap().morphisms.apply_morphism(
        rt_string.clone(),
        &Context::parse(&ctx, "<List Char>~EditTree"),
        &Context::parse(&ctx, "<List Char>")
    );

    /* Now, get the ListView that serves our char-values.
     * This view is a projection created by the morphism that was called above.
     */
    let mut chars_view = rt_string
        .read().unwrap()
        .get_port::<dyn ListView<char>>()
        .unwrap();

    /* transform ListView<char> into a TerminalView
     */
    let string_view_tty = chars_view
        .to_sequence()
        .to_grid_vertical()
        .map_item(|_pt,c| TerminalAtom::new(*c, TerminalStyle::fg_color((200,10,60))));

    /* setup terminal
     */
    let app = TTYApplication::new({
        let edittree_list = edittree_list.clone();

        /* event handler
         */
        let ctx = ctx.clone();
        move |ev| {
            edittree_list.get().send_cmd_obj(ev.to_repr_tree(&ctx));
        }
    });

    /* Setup the compositor to serve as root-view
     * by routing it to the `app.port` Viewport,
     * so it will be displayed on TTY-output.
     */
    let compositor = TerminalCompositor::new(app.port.inner());

    /* Now add some views to our compositor
     */
    {
        let mut comp = compositor.write().unwrap();

        let label_str = ctx.read().unwrap().type_term_to_str(&rt_string.read().unwrap().get_type());
        comp.push(
            nested_tty::make_label(&label_str)
                .map_item(|_pt, atom| atom.add_style_front(TerminalStyle::fg_color((90,90,90))))
                .offset(Vector2::new(1,1)));

        comp.push(
            edittree_list.get()
                .display_view()
                .offset(Vector2::new(3,2)));

        comp.push(
            string_view_tty
                .offset(Vector2::new(5,3)));
    }

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
