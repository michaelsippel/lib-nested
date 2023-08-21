use {
    r3vi::{
        view::{port::UpdateTask, OuterViewPort, singleton::*, sequence::*},
        buffer::{singleton::*, vec::*}
    },
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::list::{ListEditor, ListCursor, ListCursorMode, ListCmd, PTYListController, PTYListStyle},
        tree::{NestedNode, TreeNav, TreeCursor},
        diagnostics::Diagnostics
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_list_typename("ListCmd".into());
    ctx.add_list_typename("List".into());

    ctx.add_node_ctor(
        "List", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            let typ = args[1].clone();

                            let mut node = ListEditor::new(ctx.clone(), typ).into_node(depth);

                            PTYListController::for_node( &mut node, Some(','), Some('}') );
                            PTYListStyle::for_node( &mut node, ("{",", ","}") );

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

