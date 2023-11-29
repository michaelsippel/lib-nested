extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::{
        edit_tree::{NestedNode, TreeCursor, TreeNav},
        repr_tree::{Context, ReprTree},
        editors::ObjCommander
    },
    nested_tty::{
        terminal::TermOutWriter, DisplaySegment, Terminal, TerminalAtom, TerminalCompositor,
        TerminalEvent, TerminalStyle, TerminalView,
        TTYApplication
    },
    r3vi::{
        buffer::singleton::*,
        view::{port::UpdateTask, singleton::*, sequence::*, ViewPort},
        projection::decorate_sequence::*
    },
    std::sync::{Arc, Mutex, RwLock},
    termion::event::{Event, Key},
};

fn node_make_char_view(
    node: NestedNode
) -> NestedNode {
    node.disp.view
        .write().unwrap()
        .insert_branch(ReprTree::new_leaf(
            Context::parse(&node.ctx, "TerminalView"),
            node.data
                .read()
                .unwrap()
                .get_port::<dyn SingletonView<Item = char>>()
                .expect("unable to get Char-view")
                .map(move |c| TerminalAtom::from(if c == '\0' { ' ' } else { c }))
                .to_grid()
                .into(),
        ));

    node
}

fn node_make_seq_view(
    mut node: NestedNode
) -> NestedNode {
    node.disp.view
        .write().unwrap()
        .insert_branch(ReprTree::new_leaf(
            Context::parse(&node.ctx, "TerminalView"),
            node.data
                .read()
                .unwrap()
                .get_port::< dyn SequenceView<Item = NestedNode> >()
                .expect("unable to get Seq-view")
                .map(move |char_node| node_make_view(char_node.clone()).display_view() )
                .wrap(nested_tty::make_label("("), nested_tty::make_label(")"))
                .to_grid_horizontal()
                .flatten()
                .into()
        ));
    node
}

fn node_make_list_edit(
    mut node: NestedNode
) -> NestedNode {
    nested_tty::editors::list::PTYListStyle::for_node( &mut node, ("(", ",", ")") );
    nested_tty::editors::list::PTYListController::for_node( &mut node, None, None );

    node
}

fn node_make_view(
    node: NestedNode
) -> NestedNode {
    if node.data.read().unwrap().get_type() == &Context::parse(&node.ctx, "Char") {
        node_make_char_view( node )
    } else if node.data.read().unwrap().get_type() == &Context::parse(&node.ctx, "<Seq Char>") {
        node_make_seq_view( node )
    } else if node.data.read().unwrap().get_type() == &Context::parse(&node.ctx, "<List Char>") {
        node_make_list_edit( node )
    } else {
        eprintln!("couldnt add view");
        node
    }
}

#[async_std::main]
async fn main() {
    /* setup context & create Editor-Tree
     */
    let ctx = Arc::new(RwLock::new(Context::default()));


    /* Create a Char-Node with editor & view
     */
    let mut node1 = Context::make_node(
        &ctx,
        // node type
        Context::parse(&ctx, "Char"),
        // depth
        SingletonBuffer::new(0).get_port(),
    ).unwrap();

    // add a display view to the node
    node1 = node_make_view( node1 );

    /* Create a <List Char>-Node with editor & view
     */
    let mut node2 = Context::make_node(
        &ctx,
        // node type
        Context::parse(&ctx, "<List Char>"),
        // depth
        SingletonBuffer::new(0).get_port(),
    ).unwrap();

    // add a display view to the node
    node2 = node_make_view( node2 );

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */

        let ctx = ctx.clone();
        let mut node1 = node1.clone();
        let mut node2 = node2.clone();
        move |ev| {           
            let mut node1 = node1.clone();
            let mut node2 = node2.clone();
            node1.send_cmd_obj( ev.to_repr_tree(&ctx) );
            node2.send_cmd_obj( ev.to_repr_tree(&ctx) );
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

    let label = ctx.read().unwrap().type_term_to_str( &node1.get_type() );
    compositor.write().unwrap()
        .push(nested_tty::make_label( &label ).offset(Vector2::new(0, 2)));

    compositor.write().unwrap()
        .push(node1.display_view().offset(Vector2::new(15, 2)));


    let label2 = ctx.read().unwrap().type_term_to_str( &node2.get_type() );
    compositor.write().unwrap()
        .push(nested_tty::make_label( &label2 ).offset(Vector2::new(0, 3)));

    compositor.write().unwrap()
        .push(node2.display_view().offset(Vector2::new(15, 3)));

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
