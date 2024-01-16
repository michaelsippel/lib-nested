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

    nested::editors::char::init_ctx( ctx.clone() );
    nested::editors::integer::editor::init_ctx( ctx.clone() );
    nested::editors::list::init_ctx( ctx.clone() );

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
   let rt_digit = ReprTree::new_arc( Context::parse(&ctx, "<Digit 16>") );

    /* add initial representation
     *  <Digit 16> ~ Char
     */
    rt_digit.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "Char") ].into_iter(),
            SingletonBuffer::new('x').get_port().into()
        );

    let port_char = rt_digit.read().unwrap()
        .descend(Context::parse(&ctx, "Char")).unwrap().read().unwrap()
        .get_port::<dyn r3vi::view::singleton::SingletonView<Item = char>>().unwrap().0;

    ctx.read().unwrap()
        .morphisms
        .morph(
            rt_digit.clone(),
            &Context::parse(&ctx, "<Digit 16>~EditTree")
        );

    let port_edit = rt_digit.read().unwrap()
        .descend(Context::parse(&ctx, "EditTree")).unwrap()
        .read().unwrap()
        .get_port::<dyn r3vi::view::singleton::SingletonView<Item = Arc<RwLock<EditTree>> >>().unwrap();

    /* setup TTY-Display for DigitEditor
     */
    {
        let et = port_edit.get_view().unwrap().get().clone();
        let mut et = et.write().unwrap();
        *et = nested_tty::editors::edittree_make_digit_view(et.clone());
    }

    //---
    let rt_string = ReprTree::new_arc( Context::parse(&ctx, "<List <Digit 10>>") );
    ctx.read().unwrap()
        .morphisms
        .morph(
            rt_string.clone(),
            &Context::parse(&ctx, "<List <Digit 10>>~EditTree")
        );

    let editport_string = rt_string.read().unwrap()
        .descend(Context::parse(&ctx, "EditTree")).unwrap()
        .read().unwrap()
        .get_port::<dyn r3vi::view::singleton::SingletonView<Item = Arc<RwLock<EditTree>>> >().unwrap();

    /* setup TTY-Display for ListEditor
     */
    {
        let et = editport_string.get_view().unwrap().get().clone();
        let mut et = et.write().unwrap();
        *et = nested_tty::editors::edittree_make_list_edit(et.clone());
    }
/*
    let vec_string = Arc::new(RwLock::new(Vec::<char>::new()));
    
    rt_string.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "<Vec Char>") ].into_iter(),
            r3vi::view::ViewPort::with_view(vec_string).into_outer().into()
        );

    
    rt_string.write().unwrap()
        .insert_leaf(
            vec![ Context::parse(&ctx, "EditTree") ].into_iter(),
            r3vi::view::ViewPort::with_view()
        );
*/

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */
        let ctx = ctx.clone();
        let et1 = editport_string.clone();
        move |ev| {
            et1.get_view().unwrap().get().write().unwrap().send_cmd_obj(ev.to_repr_tree(&ctx));
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
    compositor.write().unwrap().push( port_edit.get_view().unwrap().get().read().unwrap().display_view().offset(Vector2::new(0,2)) );

    let label = ctx.read().unwrap().type_term_to_str(&rt_digit.read().unwrap().get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label).offset(Vector2::new(0, 1)));

    compositor.write().unwrap().push( editport_string.get_view().unwrap().get().read().unwrap().display_view().offset(Vector2::new(0,4)) );

    let label = ctx.read().unwrap().type_term_to_str(&rt_string.read().unwrap().get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label).offset(Vector2::new(0, 3)));

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
