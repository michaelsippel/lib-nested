extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::{
        editors::ObjCommander,
        repr_tree::{Context, ReprTree},
        edit_tree::{EditTree}
    },
    nested_tty::{
        DisplaySegment, TTYApplication,
        TerminalCompositor, TerminalStyle, TerminalView,
        TerminalAtom
    },
    r3vi::{
        buffer::{singleton::*, vec::*},
    },
    std::sync::{Arc, RwLock},
};

#[async_std::main]
async fn main() {
    /* setup context & create Editor-Tree
     */
    let ctx = Arc::new(RwLock::new(Context::new()));

    /* structure of Repr-Tree
     *
     *   === Repr-Tree ===
     *
     *        <Digit 10>
     *         /   |     \
     *        /    |        \
     *       /     |          \
     *     u32  [ EditTree ]  Char
     *           - Editor         \
     *           - Display      [ EditTree ]
     *         /     |    \      - Editor
     *        /      |     \     - Display
     *      TTY  PixelBuf  SDF  /    |     \
     *                         /     |      \
     *                       TTY  PixelBuf  SDF
     */
    let rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 10>") );
    let port_char = r3vi::view::ViewPort::<dyn r3vi::view::singleton::SingletonView<Item = char>>::new();
    let port_u32 = r3vi::view::ViewPort::<dyn r3vi::view::singleton::SingletonView<Item = u32>>::new();
    let port_edit = r3vi::view::ViewPort::<dyn r3vi::view::singleton::SingletonView<Item = EditTree>>::new();
    let port_char_edit = r3vi::view::ViewPort::<dyn r3vi::view::singleton::SingletonView<Item = EditTree>>::new();

    rt_digit.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "Char") ].into_iter(),
            port_char.outer().into()
        );

    rt_digit.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "Char"), Context::parse(&ctx, "EditTree") ].into_iter(),
            port_char_edit.outer().into()
        );

    rt_digit.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "EditTree") ].into_iter(),
            port_edit.outer().into()
        );

    rt_digit.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "u32") ].into_iter(),
            port_u32.outer().into()
        );

    /* setup projections between representations
     */

    // created by  Char  ==>  Char~EditTree
    let mut edittree_char =
        nested::editors::char::CharEditor::new_edit_tree(
            ctx.clone(),
            r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
        );

    edittree_char = nested_tty::editors::edittree_make_char_view( edittree_char );
    let mut edit_char = edittree_char.get_edit::< nested::editors::char::CharEditor >().unwrap();
    port_char.attach_to( edit_char.read().unwrap().get_port() );

    let buf_edit_char = r3vi::buffer::singleton::SingletonBuffer::new( edittree_char.clone() );
    port_char_edit.attach_to( buf_edit_char.get_port() );


    // created by   <Digit 10>   ==>  <Digit 10>~EditTree
    let mut node_edit_digit =
        nested::editors::integer::DigitEditor::new(
            ctx.clone(),
            16
        ).into_node(
            r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
        );

    node_edit_digit = nested_tty::editors::edittree_make_digit_view( node_edit_digit );
    let mut edit_digit = node_edit_digit.get_edit::< nested::editors::integer::DigitEditor >().unwrap();

    // created by   <Digit 10> ==> <Digit 10>~U32
    port_u32.attach_to(  port_char.outer().map(|c| c.to_digit(16).unwrap_or(0)) );
 //    port_u32.attach_to( edit_digit.read().unwrap().get_data_port().map(|d| d.unwrap_or(0)) );

    let port_proj_u32_to_char = port_u32.outer().map(|val| char::from_digit(val, 16).unwrap_or('?') );
    
    let buf_edit_digit = r3vi::buffer::singleton::SingletonBuffer::new( node_edit_digit );
    port_edit.attach_to( buf_edit_digit.get_port() );

/*
    ctx.write().unwrap()
        .morphisms
        .add_morphism(
            MorphismType {
                src_type: Context::parse(&ctx, "Char"),
                dst_type: Context::parse(&ctx, "Char~EditTree")
            },

            |rt, _σ| {
                rt.write().unwrap()
                    .insert_branch(
                        Context::parse(&ctx, "")
                    );
            }
        )
*/

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */
        let ctx = ctx.clone();
        let node1 = buf_edit_digit.clone();
        move |ev| {
            node1.get().send_cmd_obj(ev.to_repr_tree(&ctx));
        }
    });

    /* setup display view routed to `app.port`
     */
    let compositor = TerminalCompositor::new(app.port.inner());

    // add some views to the display compositor
    compositor.write().unwrap().push(
        nested_tty::make_label("Hello World")
            .map_item(|p, a| {
                a.add_style_back(TerminalStyle::fg_color(((25 * p.x % 255) as u8, 200, 0)))
            })
            .offset(Vector2::new(5, 0)),
    );
    compositor.write().unwrap().push( buf_edit_digit.get().display_view().offset(Vector2::new(0,2)) );

/*    let label = ctx.read().unwrap().type_term_to_str(&node1.get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label).offset(Vector2::new(0, 2)));

    compositor
        .write()
        .unwrap()
        .push(node1.display_view().offset(Vector2::new(15, 2)));
*/
    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
