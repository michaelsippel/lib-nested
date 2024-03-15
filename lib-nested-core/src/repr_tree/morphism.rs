use {
    laddertypes::{TypeTerm, TypeID},
    r3vi::view::AnyOuterViewPort,
    crate::{
        repr_tree::{ReprTree, ReprTreeExt, ReprLeaf},
    },
    std::{
        sync::{Arc, RwLock},
        collections::HashMap
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MorphismType {
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm,
}

#[derive(Clone)]
pub struct GenericReprTreeMorphism {
    morph_type: MorphismType,
    setup_projection: Arc<
        dyn Fn( &mut Arc<RwLock<ReprTree>>, &HashMap<TypeID, TypeTerm> )
//            -> Result< ReprLeaf, () >
        + Send + Sync
    >
}

#[derive(Clone)]
pub struct MorphismBase {
    morphisms: Vec< GenericReprTreeMorphism >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl MorphismBase {
    pub fn new() -> Self {
        MorphismBase {
            morphisms: Vec::new()
        }
    }

    pub fn add_morphism(
        &mut self,
        morph_type: MorphismType,
        setup_projection:
            impl Fn( &mut Arc<RwLock<ReprTree>>, &HashMap<TypeID, TypeTerm> )
//                -> Result< ReprLeaf, () /* TODO: error */ >
            + Send + Sync + 'static
    ) {
        self.morphisms.push(
            GenericReprTreeMorphism {
                morph_type,
                setup_projection: Arc::new(setup_projection)
            }
        );
    }

    pub fn find_morphism(
        &self,
        src_type: &TypeTerm,
        dst_type: &TypeTerm
    ) -> Option<(&GenericReprTreeMorphism, HashMap<TypeID, TypeTerm>)> {
        for m in self.morphisms.iter() {

            let unification_problem = laddertypes::UnificationProblem::new(
                vec![
                    ( src_type.clone(), m.morph_type.src_type.clone() ),
                    ( dst_type.clone(), m.morph_type.dst_type.clone() )
                ]
            );

            if let Ok(σ) = unification_problem.solve() {
                return Some((m, σ));
            }
        }

        None
    }

    pub fn apply_morphism(
        &self,
        mut repr_tree: Arc<RwLock<ReprTree>>,
        src_type: &TypeTerm,
        dst_type: &TypeTerm
    ) {
//        let t = repr_tree.read().unwrap().get_type().clone();
        if let Some((m, σ)) = self.find_morphism( &src_type, dst_type ) {
            (m.setup_projection)( &mut repr_tree, &σ );
        } else {
            eprintln!("could not find morphism");
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
/*
impl MorphismType {
    pub fn to_str(&self, ctx: &Context) -> String {
        format!("{:?} -> {:?}",
                if let Some(t) = self.src_type.as_ref() {
                    ctx.type_dict.read().unwrap().unparse(t)
                } else {
                    "None".into()
                },
                ctx.type_dict.read().unwrap().unparse(&self.dst_type))
    }
}
*/
