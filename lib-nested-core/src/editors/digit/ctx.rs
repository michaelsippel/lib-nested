
use {
    laddertypes::TypeTerm,
    r3vi::{
        buffer::singleton::SingletonBuffer,
        view::{
            AnyOuterViewPort,
            singleton::*
        }
    },
    crate::{
        repr_tree::{Context, ReprTree, ReprTreeExt, ReprLeaf},
        editors::digit::DigitEditor,
    },
    std::sync::{Arc, RwLock}
};

pub fn init_ctx( ctx: Arc<RwLock<Context>> ) {

    // todo: proper scoping of Radix variable
    ctx.write().unwrap().add_varname("Radix");

    let morphtype =
            crate::repr_tree::MorphismType {
                src_type: Context::parse(&ctx, "<Digit Radix>"),
                dst_type: Context::parse(&ctx, "<Digit Radix>~EditTree")
            };

    ctx.write().unwrap()
        .morphisms
        .add_morphism(
            morphtype,
            {
                let ctx = ctx.clone();
                move |src_rt, σ| {
                    let radix =
                        match σ.get( &laddertypes::TypeID::Var(ctx.read().unwrap().get_var_typeid("Radix").unwrap()) ) {
                            Some(TypeTerm::Num(n)) => *n as u32,
                            _ => 0
                        };

                    /* get char representation or create it if not available
                     */
                    let char_rt =
                        if let Some(crt) = src_rt.descend(Context::parse(&ctx, "Char")) {
                            crt
                        } else {
                            let crt = ReprTree::from_singleton_buffer(
                                Context::parse(&ctx, "Char"),
                                SingletonBuffer::new('\0')
                            );
                            src_rt.insert_branch(crt.clone());
                            crt
                        };

                    /* Create EditTree object
                     */
                    let mut edittree = DigitEditor::new(
                        ctx.clone(),
                        radix,
                        src_rt.descend(
                            Context::parse(&ctx, "Char")
                        ).unwrap()
                        .singleton_buffer::<char>()
                    ).into_node(
                        r3vi::buffer::singleton::SingletonBuffer::<usize>::new(0).get_port()
                    );

                    src_rt.write().unwrap()
                        .insert_branch(
                            ReprTree::from_singleton_buffer(
                                Context::parse(&ctx, "EditTree"),
                                SingletonBuffer::new(edittree)
                            )
                        );
                }
            }
        );

    let morphtype =
            crate::repr_tree::MorphismType {
                src_type: Context::parse(&ctx, "<Digit Radix>~Char"),
                dst_type: Context::parse(&ctx, "<Digit Radix>~ℤ_256~machine::UInt8")
            };

    ctx.write().unwrap()
        .morphisms
        .add_morphism(
            morphtype,
            {
                let ctx = ctx.clone();
                move |rt: &mut Arc<RwLock<ReprTree>>, σ: &std::collections::HashMap<laddertypes::TypeID, TypeTerm>| {
                    /* infer radix from type
                     */
                    let radix =
                        match σ.get( &laddertypes::TypeID::Var(ctx.read().unwrap().get_var_typeid("Radix").unwrap()) ) {
                            Some(TypeTerm::Num(n)) => (*n) as u32,
                            _ => 0
                        };

                    if radix <= 256 {

                        if let Some(src_rt) = rt.descend(Context::parse(&ctx, "Char")) {
                            /* insert projected view into ReprTree
                             */
                            let u8_view = 
                                    src_rt.view_char()
                                        .map(move |c| c.to_digit(radix).unwrap_or(0) as u8);

                            rt.write().unwrap().attach_leaf_to::<dyn SingletonView<Item = u8>>(
                                Context::parse(&ctx, "ℤ_256~machine::UInt8").get_lnf_vec().into_iter(),
                                u8_view
                            );
                        } else {
                            eprintln!("could not find required source representation: <Digit {}>~Char", radix);
                        }
                    } else {
                        eprintln!("radix too large ({})", radix);
                    }
                }
            }
        );


    let morphtype =
            crate::repr_tree::MorphismType {
                src_type: Context::parse(&ctx, "<Digit Radix>~ℤ_256~machine::UInt8"),
                dst_type: Context::parse(&ctx, "<Digit Radix>~Char")
            };

    ctx.write().unwrap().morphisms
        .add_morphism(morphtype, {
            let ctx = ctx.clone();
            move |rt: &mut Arc<RwLock<ReprTree>>, σ: &std::collections::HashMap<laddertypes::TypeID, TypeTerm>| {
                /* infer radix from type
                 */
                let radix =
                    match σ.get( &laddertypes::TypeID::Var(ctx.read().unwrap().get_var_typeid("Radix").unwrap()) ) {
                       Some(TypeTerm::Num(n)) => (*n) as u32,
                        _ => 0
                    };

                if radix <= 256 {
                    /* insert projected view into ReprTree
                     */
                    let char_view = 
                        rt.descend(Context::parse(&ctx, "ℤ_256~machine::UInt8"))
                            .unwrap()
                            .view_u8()
                            .map(move |digit| char::from_digit(digit as u32, radix).unwrap_or('?'));

                    rt.write().unwrap().attach_leaf_to::<dyn SingletonView<Item = char>>(
                        Context::parse(&ctx, "Char").get_lnf_vec().into_iter(),
                        char_view
                    );
                } else {
                    eprintln!("radix too large ({})", radix);
                }
            }
        });
}


