use {
    crate::{
        type_system::{Context, TypeTerm, ReprTree},
        editors::{
            char::*,
            list::*,
            integer::*,
            product::*
        },
        tree::{NestedNode},
        diagnostics::{Diagnostics},
        type_system::{MorphismTypePattern},
    },
    std::sync::{Arc, RwLock},
    cgmath::Point2
};

pub fn init_mem_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ctx.write().unwrap().add_node_ctor(
        "Vec", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: usize| {
                match ty {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            let buf = r3vi::buffer::vec::VecBuffer::<char>::new();
                            let data = ReprTree::new_leaf(
                                ctx.read().unwrap().type_term_from_str("( Char )").unwrap(),
                                buf.get_port().into()
                            );

                            Some(
                                NestedNode::new(ctx, data, depth)
                                    .set_editor(Arc::new(RwLock::new(buf)))
                            )
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );

    ctx
}

pub fn init_editor_ctx(parent: Arc<RwLock<Context>>) -> Arc<RwLock<Context>> {
    let ctx0 = Arc::new(RwLock::new(Context::with_parent(Some(parent))));

    ListEditor::init_ctx( &ctx0 );
    
    let mut ctx = ctx0.write().unwrap();
    // TODO:: CharEditor::init_ctx( &ctx );

    ctx.add_node_ctor(
        "Char", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, _depth: usize| {
                Some(CharEditor::new_node(ctx))
            }
        )
    );

    ctx.add_list_typename("Seq".into());
    ctx.add_list_typename("Sequence".into());
    ctx.add_list_typename("SepSeq".into());
    ctx.add_typename("NestedNode".into());

    ctx.add_list_typename("Symbol".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("Symbol").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, None );
                PTYListStyle::for_node( &mut node, ("","","") );

                Some(node)
            }
        )
    );

    ctx.add_node_ctor(
        "Symbol", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    (&ctx, "( List Char )").into(),
                    depth+1
                ).expect("nested node");

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );

    ctx.add_list_typename("String".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("String").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, _dst_type:_| {
                PTYListController::for_node( &mut node, None, Some('\"') );
                PTYListStyle::for_node( &mut node, ("\"","","\"") );
                Some(node)                
            }
        )
    );

   ctx.add_node_ctor(
        "String", Arc::new(
            |ctx: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: usize| {
                let mut node = Context::make_node(
                    &ctx,
                    TypeTerm::App(vec![
                        TypeTerm::TypeID(ctx.read().unwrap().get_typeid("List").unwrap()),
                        TypeTerm::new(ctx.read().unwrap().get_typeid("Char").unwrap())
                    ]),
                    depth+1
                ).unwrap();

                node = node.morph(dst_typ);

                Some(node)
            }
        )
    );
/*
    ctx.add_list_typename("TypeTerm".into());
    ctx.add_node_ctor(
        "TypeTerm", Arc::new(
            |ctx: Arc<RwLock<Context>>, _ty: TypeTerm, depth: usize| {
                Some(TypeTermEditor::new(ctx, depth).into_node(depth))
            }
        )
    );
*/
    ctx.add_typename("TerminalEvent".into());

    drop(ctx);
    ctx0
}


