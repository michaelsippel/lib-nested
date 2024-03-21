//! This example demonstrates how a very simple editor for hexadecimal digits
//! can be created with `lib-nested` and the `lib-nested-tty` backend.

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
    /* setup context & create Editor-Tree
     */
    let ctx = Arc::new(RwLock::new(Context::new()));

    nested::editors::char::init_ctx( ctx.clone() );
    nested::editors::digit::init_ctx( ctx.clone() );
    nested::editors::integer::init_ctx( ctx.clone() );
    nested::editors::list::init_ctx( ctx.clone() );

    nested_tty::setup_edittree_hook(&ctx);

    /* structure of Repr-Tree
     *
     *   === Repr-Tree ===
     *
     *        <Digit 10>
     *         /   |     \
     *        /    |        \
     *       /     |          \
     *     u32   EditTree     Char
     *           - Editor         \
     *           - Display        EditTree
     *         /     |    \      - Editor
     *        /      |     \     - Display
     *      TTY  PixelBuf  SDF  /    |     \
     *                         /     |      \
     *                       TTY  PixelBuf  SDF
     */
    let mut rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );

    /* add initial representation
     *  <Digit 16> ~ Char
     */
    rt_digit.insert_leaf(
        Context::parse(&ctx, "Char"),
        nested::repr_tree::ReprLeaf::from_singleton_buffer( SingletonBuffer::new('5') )
    );

    /* furthermore, setup projections to and from u8 value,
     * this synchronizes the buffers 
     */
    ctx.read().unwrap().morphisms.apply_morphism(
        rt_digit.clone(),
        &Context::parse(&ctx, "<Digit 16>~Char"),
        &Context::parse(&ctx, "<Digit 16>~ℤ_256~machine::UInt8")
    );

    /* setup TTY-Display for DigitEditor
     *
     * `setup_edittree` will setup the projection
     *    Char -> Char~EditTree
     *  and call the hook defined above with `set_edittree_hook()`
     *
     */
    let edittree_digit = ctx.read().unwrap()
        .setup_edittree(
            rt_digit.clone(),
            SingletonBuffer::new(0).get_port()
        );

    let mut digit_u8_buffer = rt_digit
        .descend(Context::parse(&ctx, "ℤ_256~machine::UInt8")).unwrap()
        .singleton_buffer::<u8>();

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */
        let ctx = ctx.clone();

        let mut edittree_digit = edittree_digit.clone();
        move |ev| {
            edittree_digit.get().send_cmd_obj(ev.to_repr_tree(&ctx));
        }
    });

    /* setup display view routed to `app.port`
     */
    let compositor = TerminalCompositor::new(app.port.inner());

    // add some views to the display compositor
    {
        let mut comp = compositor.write().unwrap();

        comp.push(
            nested_tty::make_label("Hello World")
                .map_item(|p, a| {
                    a.add_style_back(TerminalStyle::fg_color(((25 * p.x % 255) as u8, 200, 0)))
                })
                .offset(Vector2::new(5, 0)),
        );

        let label_str = ctx.read().unwrap().type_term_to_str(&rt_digit.read().unwrap().get_type());
        comp.push(
            nested_tty::make_label(&label_str)
            .map_item(|_pt,atom| atom.add_style_front(TerminalStyle::fg_color((90,90,90))))
            .offset(Vector2::new(1, 1))
        );
        comp.push(
            edittree_digit.get().display_view()
            .offset(Vector2::new(3,2))
        );
        comp.push(
            digit_u8_buffer.get_port().map(
                |d| nested_tty::make_label(&format!("Digit value={}", d))
            )
            .to_grid()
            .flatten()
            .offset(Vector2::new(5,3))
        );
    }

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
