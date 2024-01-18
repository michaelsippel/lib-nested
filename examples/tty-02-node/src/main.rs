extern crate cgmath;
extern crate nested;
extern crate nested_tty;
extern crate r3vi;
extern crate termion;

use {
    cgmath::Vector2,
    nested::{
        editors::ObjCommander,
        repr_tree::{Context},
    },
    nested_tty::{
        DisplaySegment, TTYApplication,
        TerminalCompositor, TerminalStyle, TerminalView,
    },
    r3vi::{
        buffer::singleton::*,
    },
    std::sync::{Arc, RwLock},
};

struct ParseDigit { radix: u32 };
impl Morphism for ParseDigit {
    fn new(
        ctx: &Arc<RwLock<Context>>
    ) -> Self {
        
    }

    fn setup_projection(&self, repr_tree: Arc<RwLock<ReprTree>>) {
        if let Some( char_view ) = repr_tree.get_out(Context::parse(&ctx, "Char~")) {
            
        }
    }
}

get_morphism( ) -> Morphism {
    
}

#[async_std::main]
async fn main() {
    /* setup context & create Editor-Tree
     */
    let ctx = Arc::new(RwLock::new(Context::default()));

    /* Create a Char-Node with editor & view
     */
    let mut char_obj = ReprTree::make_leaf(
        Context::parse(&ctx, "Char"),
        SingletonBuffer::new('X').get_port().into()
    );

    char_obj.insert_branch(
        Context::parse(&ctx, "EditTree"),
        SingletonBuffer::new(
            
        )
    );

    let mut vec_obj = ReprTree::make_leaf(
        Context::parse(&ctx, "<Vec Char>"),
        VecBuffer::new(vec!['a', 'b', 'c']).get_port().into()
    );

    let mut char_edit = Context::new_edit_tree(
        &ctx,
        // node type
        Context::parse(&ctx, "Char"),
        // depth
        SingletonBuffer::new(0).get_port(),
    )
    .unwrap();

    // add a display view to the node
    node1 = nested_tty::editors::node_make_tty_view(node1);

    /* Create a <List Char>-Node with editor & view
     */
    let mut node2 = Context::make_node(
        &ctx,
        // node type
        Context::parse(&ctx, "<List Char>"),
        // depth
        SingletonBuffer::new(0).get_port(),
    )
    .unwrap();

    // add a display view to the node
    node2 = nested_tty::editors::node_make_tty_view(node2);

    /* Create a <List Char>-Node with editor & view
     */
    let mut node3 = Context::make_node(
        &ctx,
        // node type
        Context::parse(&ctx, "<List <List Char>>"),
        // depth
        SingletonBuffer::new(0).get_port(),
    )
    .unwrap();

    // add a display view to the node
    node3 = nested_tty::editors::node_make_tty_view(node3);

    /* setup terminal
     */
    let app = TTYApplication::new({
        /* event handler
         */

        let ctx = ctx.clone();
        let node1 = node1.clone();
        let node2 = node2.clone();
        let node3 = node3.clone();
        move |ev| {
            let mut node1 = node1.clone();
            let mut node2 = node2.clone();
            let mut node3 = node3.clone();
            node1.send_cmd_obj(ev.to_repr_tree(&ctx));
            node2.send_cmd_obj(ev.to_repr_tree(&ctx));
            node3.send_cmd_obj(ev.to_repr_tree(&ctx));
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

    let label = ctx.read().unwrap().type_term_to_str(&node1.get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label).offset(Vector2::new(0, 2)));

    compositor
        .write()
        .unwrap()
        .push(node1.display_view().offset(Vector2::new(15, 2)));

    let label2 = ctx.read().unwrap().type_term_to_str(&node2.get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label2).offset(Vector2::new(0, 3)));

    compositor
        .write()
        .unwrap()
        .push(node2.display_view().offset(Vector2::new(15, 3)));


    let label3 = ctx.read().unwrap().type_term_to_str(&node3.get_type());
    compositor
        .write()
        .unwrap()
        .push(nested_tty::make_label(&label3).offset(Vector2::new(0, 4)));

    compositor
        .write()
        .unwrap()
        .push(node3.display_view().offset(Vector2::new(25, 4)));

    /* write the changes in the view of `term_port` to the terminal
     */
    app.show().await.expect("output error!");
}
