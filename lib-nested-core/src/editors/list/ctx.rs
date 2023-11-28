use {
    r3vi::{view::{OuterViewPort, singleton::*}},
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context},
        editors::list::{ListEditor}//, PTYListController, PTYListStyle}
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_typename("ListCmd".into());
    ctx.add_list_typename("List".into());

    ctx.add_node_ctor(
        "List", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: OuterViewPort<dyn SingletonView<Item = usize>>| {
                match ty {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            let typ = args[1].clone();

                            let mut node = ListEditor::new(ctx.clone(), typ).into_node(depth);

//                            PTYListController::for_node( &mut node, Some(','), Some('}') );
//                            PTYListStyle::for_node( &mut node, ("{",", ","}") );

                            Some(node)
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );
}

