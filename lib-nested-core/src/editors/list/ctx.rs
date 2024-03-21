use {
    r3vi::{
        view::{
            ViewPort,
            OuterViewPort, Observer, singleton::*
        },
        buffer::{singleton::*, vec::*}
    },
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context, ReprTree, ReprLeaf, ReprTreeExt},
        edit_tree::{EditTree},
        editors::{
            char::{CharEditor},
            list::{ListEditor}//, PTYListController, PTYListStyle}
        }
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
                        SingletonBuffer::<usize>::new(0).get_port()
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

    let mt = crate::repr_tree::MorphismType {
        src_type: Context::parse(&ctx, "<List Char>~EditTree"),
        dst_type: Context::parse(&ctx, "<List Char>")
    };
    ctx.write().unwrap().morphisms.add_morphism(
        mt,
        {
            let ctx = ctx.clone();
            move |src_rt, σ| {
                let edittree =
                    src_rt
                        .descend(Context::parse(&ctx, "EditTree")).unwrap()
                        .singleton_buffer::<EditTree>();

                let list_edit = edittree.get().get_edit::< ListEditor >().unwrap();
                let edittree_items = list_edit.read().unwrap().data.get_port().to_list();
                src_rt.write().unwrap().insert_leaf(
                    vec![].into_iter(),
                    ReprLeaf::from_view(
                        edittree_items
                            .map(
                                |edittree_char|
                                    edittree_char
                                    .read().unwrap()
                                    .get_edit::<CharEditor>().unwrap()
                                    .read().unwrap()
                                    .get()
                            )
                    )
                );
            }
        }
    );
}

