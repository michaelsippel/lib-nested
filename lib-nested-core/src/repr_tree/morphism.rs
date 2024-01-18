use {
    laddertypes::{TypeTerm, TypeID},
    crate::{
        repr_tree::{ReprTree},
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
    repr_tree_op: Arc<
        dyn Fn( Arc<RwLock<ReprTree>>, &HashMap<TypeID, TypeTerm> )
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
        repr_tree_op: impl Fn( Arc<RwLock<ReprTree>>, &HashMap<TypeID, TypeTerm> ) + Send + Sync
    ) {
        self.morphisms.push(
            GenericReprTreeMorphism {
                morph_type,
                repr_tree_op: Arc::new(repr_tree_op)
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
                eprintln!("found matching morphism");
                return Some((m, σ));
            }
        }

        None
    }

    pub fn morph(
        &self,
        repr_tree: Arc<RwLock<ReprTree>>,
        target_type: &TypeTerm
    ) {
        if let Some((m, σ)) = self.find_morphism( repr_tree.read().unwrap().get_type(), target_type ) {
            (m.repr_tree_op)( repr_tree, &σ );
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
