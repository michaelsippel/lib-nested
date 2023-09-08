
use {
    r3vi::{
        view::{OuterViewPort, singleton::*}
    },
    crate::{
        type_system::{Context, TypeTerm},
        editors::{
            list::*,
            integer::*
        },
        type_system::{MorphismTypePattern},
    },
    std::sync::{Arc, RwLock}
};

pub fn init_ctx(ctx: &mut Context) {
    ctx.add_typename("MachineInt".into());
    ctx.add_typename("u32".into());
    ctx.add_typename("u64".into());
    ctx.add_typename("LittleEndian".into());
    ctx.add_typename("BigEndian".into());

    ctx.add_node_ctor(
        "Digit", Arc::new(
            |ctx: Arc<RwLock<Context>>, ty: TypeTerm, depth: OuterViewPort<dyn SingletonView<Item = usize>>| {
                match ty {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
                                TypeTerm::Num(radix) => {
                                    let node = DigitEditor::new(ctx.clone(), radix as u32).into_node(depth);
                                    Some(
                                        node
                                    )
                                },
                                _ => None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );

    ctx.add_list_typename("PosInt".into());
    let pattern = MorphismTypePattern {
        src_tyid: ctx.get_typeid("List"),
        dst_tyid: ctx.get_typeid("PosInt").unwrap()
    };
    ctx.add_morphism(pattern,
        Arc::new(
            |mut node, dst_type| {
                // todo: check src_type parameter to be ( Digit radix )

                match dst_type {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
                                TypeTerm::Num(_radix) => {
                                    PTYListController::for_node(
                                        &mut node,
                                        Some(','),
                                        None,
                                    );

                                    PTYListStyle::for_node(
                                        &mut node,
                                        ("0d", "", "")
                                    );

                                    Some(node)
                                },
                                _ => None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );

    ctx.add_node_ctor(
        "PosInt", Arc::new(
            |ctx0: Arc<RwLock<Context>>, dst_typ: TypeTerm, depth: OuterViewPort<dyn SingletonView<Item = usize>>| {
                match dst_typ.clone() {
                    TypeTerm::App(args) => {
                        if args.len() > 1 {
                            match args[1] {
                                TypeTerm::Num(radix) => {
                                    let ctx = ctx0.read().unwrap();
                                    let mut node = Context::make_node(
                                        &ctx0,
                                        TypeTerm::App(vec![
                                            TypeTerm::TypeID(ctx.get_typeid("List").unwrap()),
                                            TypeTerm::TypeID(
                                                ctx.get_typeid("Digit").unwrap()
                                            )
                                                .num_arg(radix)
                                                .clone()
                                                .into()
                                        ]),
                                        depth.map(|d| d+1)
                                    ).unwrap();

                                    node = node.morph(dst_typ);

                                    Some(node)
                                }
                                _ => None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            }
        )
    );
    
    ctx.add_typename("Date".into());
    ctx.add_typename("ISO-8601".into());
    ctx.add_typename("TimeSince".into());
    ctx.add_typename("UnixEpoch".into());
    ctx.add_typename("AnnoDomini".into());
    ctx.add_typename("Epoch".into());
    ctx.add_typename("Duration".into());
    ctx.add_typename("Seconds".into());
    ctx.add_typename("ℕ".into());
}

