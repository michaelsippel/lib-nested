use {
    r3vi::{view::{OuterViewPort, singleton::*}, buffer::singleton::*},
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context, ReprTree},
        editors::list::{ListEditor}//, PTYListController, PTYListStyle}
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub fn init_ctx(ctx: Arc<RwLock<Context>>) {
    ctx.write().unwrap().add_list_typename("List".into());
    ctx.write().unwrap().add_varname("Item");

    let mt = crate::repr_tree::MorphismType {
        src_type: Context::parse(&ctx, "<List Item>"),
        dst_type: Context::parse(&ctx, "<List Item>~EditTree")
    };
    ctx.write().unwrap().morphisms.add_morphism(
        mt,
        {
            let ctx = ctx.clone();
            move |src_rt, σ| {
                let item_id = laddertypes::TypeID::Var( ctx.read().unwrap().get_var_typeid("Item").unwrap() );
                if let Some( item_type ) = σ.get( &item_id ) {

                    let mut edittree_list = ListEditor::new(
                        ctx.clone(),
                        item_type.clone()
                    ).into_node(
                        r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
                    );

                    src_rt.write().unwrap().insert_branch(
                        ReprTree::from_singleton_buffer(
                            Context::parse(&ctx, "EditTree"),
                            SingletonBuffer::new(edittree_list)
                        )
                    );
                } else {
                    eprintln!("no item type");
                }
            }
        }
    );
}

