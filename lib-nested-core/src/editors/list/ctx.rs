use {
    r3vi::{view::{OuterViewPort, singleton::*}, buffer::singleton::*},
    laddertypes::{TypeTerm},
    crate::{
        repr_tree::{Context},
        editors::list::{ListEditor}//, PTYListController, PTYListStyle}
    },
    std::sync::{Arc, RwLock}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub fn init_ctx(ctx: Arc<RwLock<Context>>) {

    ctx.write().unwrap().add_varname("Item");
    let mt = crate::repr_tree::MorphismType {
        src_type: Context::parse(&ctx, "<List Item>"),
        dst_type: Context::parse(&ctx, "<List Item>~EditTree")
    };
    ctx.write().unwrap().morphisms.add_morphism(
        mt,
        {
            let ctx = ctx.clone();
            move |rt, σ| {
                let item_id = laddertypes::TypeID::Var( ctx.read().unwrap().get_var_typeid("Item").unwrap() );
                if let Some( item_type ) = σ.get( &item_id ) {
                    eprintln!("create list of {:?}", item_type);
                    let mut edittree_list = ListEditor::new(
                        ctx.clone(),
                        item_type.clone()
                    ).into_node(
                        r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
                    );

                    let mut rt = rt.write().unwrap();
                    rt.insert_leaf(
                        vec![ Context::parse(&ctx, "EditTree") ].into_iter(),
                        SingletonBuffer::new( Arc::new(RwLock::new( edittree_list )) ).get_port().into()
                    );
                } else {
                    eprintln!("no item type");
                }
            }
        }
    );
/*
    
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
    */
}

