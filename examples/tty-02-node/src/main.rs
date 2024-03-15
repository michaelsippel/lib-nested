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
        view::{port::UpdateTask}
    },
//    termion::{},
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

    let char_type = Context::parse(&ctx, "Char");
    let digit_type = Context::parse(&ctx, "<Digit Radix>");
    let list_type = Context::parse(&ctx, "<List Item>");
    let posint_type = Context::parse(&ctx, "<PosInt Radix>");
    let item_tyid = ctx.read().unwrap().get_var_typeid("Item").unwrap();

    ctx.write().unwrap().meta_chars.push(',');
    ctx.write().unwrap().meta_chars.push('\"');
    ctx.write().unwrap().meta_chars.push('}');

    // Define a hook which is executed when a new editTree of type `t` is created.
    // this will setup the display and navigation elements of the editor.
    // It provides the necessary bridge to the rendering- & input-backend.
    ctx.write().unwrap().set_edittree_hook(
        Arc::new(
            move |et: &mut EditTree, t: laddertypes::TypeTerm| {
//                let mut et = et.write().unwrap();

                if let Ok(σ) = laddertypes::unify(&t, &char_type.clone()) {
                    *et = nested_tty::editors::edittree_make_char_view(et.clone());
                }
                else if let Ok(σ) = laddertypes::unify(&t, &digit_type) {
                    *et = nested_tty::editors::edittree_make_digit_view(et.clone());
                }
                else if let Ok(σ) = laddertypes::unify(&t, &posint_type) {
                    nested_tty::editors::list::PTYListStyle::for_node( &mut *et, ("0d", "", ""));
                    nested_tty::editors::list::PTYListController::for_node( &mut *et, None, None );
                }
                else if let Ok(σ) = laddertypes::unify(&t, &list_type) {
                    let item_type = σ.get( &laddertypes::TypeID::Var(item_tyid) ).unwrap();

                    // choose style based on element type
                    if *item_type == char_type {
                        nested_tty::editors::list::PTYListStyle::for_node( &mut *et, ("\"", "", "\""));
                        nested_tty::editors::list::PTYListController::for_node( &mut *et, None, Some('\"') );
                    } else {
                        nested_tty::editors::list::PTYListStyle::for_node( &mut *et, ("{", ", ", "}"));
                        nested_tty::editors::list::PTYListController::for_node( &mut *et, Some(','), Some('}') );
                    }
                    //*et = nested_tty::editors::edittree_make_list_edit(et.clone());
                }
            }
        )
    );

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
            r3vi::buffer::singleton::SingletonBuffer::new(0).get_port()
        );

    let mut digit_u8_buffer = rt_digit
        .descend(Context::parse(&ctx, "ℤ_256~machine::UInt8")).unwrap()
        .singleton_buffer::<u8>();

    //---
    let rt_string = ReprTree::new_arc( Context::parse(&ctx, "<List Char>") );

    let edittree_list = ctx.read().unwrap()
        .setup_edittree(
            rt_string.clone(),
            r3vi::buffer::singleton::SingletonBuffer::new(0).get_port());

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */
        let ctx = ctx.clone();
        let mut editors = Vec::new();
        editors.push(edittree_digit.clone());
        editors.push(edittree_list.clone());

        let edit_select = Arc::new(RwLock::new(0));
        move |ev| {
            match ev {
                TerminalEvent::Input(termion::event::Event::Key(termion::event::Key::Char('\t'))) => {
                    let mut i = edit_select.write().unwrap();
                    *i = (*i + 1) % editors.len();
                }
                _ => {
                    let i = edit_select.read().unwrap();
                    editors[*i].get().send_cmd_obj(ev.to_repr_tree(&ctx));
                }
            }
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

        comp.push(
            edittree_digit.get().display_view()
            .offset(Vector2::new(2,2))
        );

        let label_str = ctx.read().unwrap().type_term_to_str(&rt_digit.read().unwrap().get_type());
        comp.push(
            nested_tty::make_label(&label_str)
            .map_item(|_pt,atom| atom.add_style_front(TerminalStyle::fg_color((90,90,90))))
            .offset(Vector2::new(0, 1))
        );

        comp.push(
            digit_u8_buffer.get_port().map(
                |d| nested_tty::make_label(&format!("Digit={}", d))
            )
            .to_grid()
            .flatten()
            .offset(Vector2::new(2,3))
        );


        comp.push(
            edittree_list.get().display_view()
            .offset(Vector2::new(2,6))
        );

        let label_str = ctx.read().unwrap().type_term_to_str(&rt_string.read().unwrap().get_type());
        comp.push(
            nested_tty::make_label(&label_str)
            .map_item(|_pt, atom| atom.add_style_front(TerminalStyle::fg_color((90,90,90))))
            .offset(Vector2::new(0, 5))
        );
    }

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
