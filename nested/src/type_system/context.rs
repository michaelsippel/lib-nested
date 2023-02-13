use {
    crate::{
        type_system::{TypeDict, TypeTerm, TypeID, ReprTree},
        tree::NestedNode
    },
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TypeLadder(Vec<TypeTerm>);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum MorphismMode {
    /// Isomorphism
    /// e.g. `( PositionalInteger 10 BigEndian ) <~> ( PositionalInteger 16 LittleEndian )`
    Iso,

    /// Monomorphism, i.e. injective functions,
    /// upcast-view, downcast-control, semantic gain
    /// e.g. `( Sequence ( Digit 16 ) ) ~> ( PositionalInteger 16 LittleEndian )`
    Mono,

    /// Epimorphsim, i.e. surjective functions,
    /// upcast-control, downcast-view, possible loss of entropy
    /// e.g. `( Ascii ) ~> ( Digit 16 )`
    Epi,

    /// Any other function
    Any,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct MorphismType {
    pub mode: MorphismMode,
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Context {
    /// assigns a name to every type
    type_dict: Arc<RwLock<TypeDict>>,

    /// objects
    objects: HashMap<String, Arc<RwLock<ReprTree>>>,

    /// types that can be edited as lists
    list_types: Vec<TypeID>,

    /// editors
    editor_ctors: HashMap<TypeID, Arc<dyn Fn(Arc<RwLock<Self>>, TypeTerm, usize) -> Option<NestedNode> + Send + Sync>>,

    /// morphisms
    default_constructors: HashMap<TypeTerm, Box<dyn Fn() -> Arc<RwLock<ReprTree>> + Send + Sync>>,
    morphism_constructors: HashMap<MorphismType, Box<dyn Fn(Arc<RwLock<ReprTree>>) -> Arc<RwLock<ReprTree>> + Send + Sync>>,

    /// recursion
    parent: Option<Arc<RwLock<Context>>>,
}

impl Context {
    pub fn with_parent(parent: Option<Arc<RwLock<Context>>>) -> Self {
        Context {
            type_dict: match parent.as_ref() {
                Some(p) => p.read().unwrap().type_dict.clone(),
                None => Arc::new(RwLock::new(TypeDict::new()))
            },
            editor_ctors: HashMap::new(),
            default_constructors: HashMap::new(),
            morphism_constructors: HashMap::new(),
            objects: HashMap::new(),
            list_types: match parent.as_ref() {
                Some(p) => p.read().unwrap().list_types.clone(),
                None => Vec::new()
            },
            parent,
        }
    }

    pub fn new() -> Self {
        Context::with_parent(None)
    }

    pub fn add_typename(&mut self, tn: String) -> TypeID {
        self.type_dict.write().unwrap().add_typename(tn)
    }

    pub fn add_list_typename(&mut self, tn: String) {
        let tid = self.add_typename(tn);
        self.list_types.push( tid );
    }

    pub fn is_list_type(&self, t: &TypeTerm) -> bool {
        match t {
            TypeTerm::Type { id, args: _ } => {
                self.list_types.contains(id)
            }
            _ => false
        }
    }

    pub fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.type_dict.read().unwrap().get_typeid(&tn.into())
    }

    pub fn type_term_from_str(&self, tn: &str) -> Option<TypeTerm> {
        self.type_dict.read().unwrap().type_term_from_str(&tn)
    }

    pub fn type_term_to_str(&self, t: &TypeTerm) -> String {
        self.type_dict.read().unwrap().type_term_to_str(&t)
    }

    pub fn add_editor_ctor(&mut self, tn: &str, mk_editor: Arc<dyn Fn(Arc<RwLock<Self>>, TypeTerm, usize) -> Option<NestedNode> + Send + Sync>) {
        let mut dict = self.type_dict.write().unwrap();
        let tyid =
            if let Some(tyid) = dict.get_typeid(&tn.into()) {
                tyid
            } else {
                dict.add_typename(tn.into())
            };
        self.editor_ctors.insert(tyid, mk_editor);
    }

    pub fn get_editor_ctor(&self, ty: &TypeTerm) -> Option<Arc<dyn Fn(Arc<RwLock<Self>>, TypeTerm, usize) -> Option<NestedNode> + Send + Sync>> {
        if let TypeTerm::Type{ id, args: _ } = ty.clone() {
            if let Some(m) = self.editor_ctors.get(&id).cloned() {
                Some(m)
            } else {
                self.parent.as_ref()?
                    .read().unwrap()
                    .get_editor_ctor(&ty)
            }
        } else {
            None
        }
    }

    pub fn make_editor(ctx: &Arc<RwLock<Self>>, type_term: TypeTerm, depth: usize) -> Option<NestedNode> {
        let mk_editor = ctx.read().unwrap().get_editor_ctor(&type_term)?;
        mk_editor(ctx.clone(), type_term, depth)
    }
/*
    pub fn enrich_editor(
        node: NestedNode,
        typ: TypeTerm
    ) -> NestedNode {

        
        // create view

        // create commander
        

    }
*/
    pub fn add_morphism(
        &mut self,
        morph_type: MorphismType,
        morph_fn: Box<dyn Fn(Arc<RwLock<ReprTree>>) -> Arc<RwLock<ReprTree>> + Send + Sync>,
    ) {
        self.morphism_constructors.insert(morph_type, morph_fn);
    }

    /// adds an object without any representations
    pub fn add_obj(&mut self, name: String, typename: &str) {
        let type_tag = self.type_dict.read().unwrap().type_term_from_str(typename).unwrap();

        self.objects.insert(
            name,
            if let Some(ctor) = self.default_constructors.get(&type_tag) {
                ctor()
            } else {
                Arc::new(RwLock::new(ReprTree::new(type_tag)))
            },
        );
    }

    pub fn get_obj(&self, name: &String) -> Option<Arc<RwLock<ReprTree>>> {
        if let Some(obj) = self.objects.get(name) {
            Some(obj.clone())
        } else if let Some(parent) = self.parent.as_ref() {
            parent.read().unwrap().get_obj(name)
        } else {
            None
        }
    }

/*
    pub fn get_obj_port<'a, V: View + ?Sized + 'static>(
        &self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>,
    ) -> Option<OuterViewPort<V>>
    where
        V::Msg: Clone,
    {
        self.get_obj(&name.into())?
            .downcast_ladder(type_ladder.map(|tn| self.type_dict.type_term_from_str(tn).unwrap()))?
            .get_port()
    }

    pub fn insert_repr<'a>(
        &mut self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>,
        port: AnyOuterViewPort,
    ) {
        self.get_obj(&name.to_string())
            .unwrap()
            .repr
            .write()
            .unwrap()
            .insert_leaf(
                type_ladder.map(|tn| self.type_dict.type_term_from_str(tn).unwrap()),
                port,
            );
    }

    pub fn epi_cast(&mut self, name: &str, typename: &str) {
        let dst_type = self.type_dict.type_term_from_str(typename).unwrap();
        let old_obj = self.objects.get(&name.to_string()).unwrap().clone();
        let new_obj = if let Some(ctor) = self.morphism_constructors.get(&MorphismType {
            mode: MorphismMode::Epi,
            src_type: old_obj.type_tag.clone(),
            dst_type: dst_type.clone(),
        }) {
            ctor(old_obj.clone())
        } else {
            Arc::new(RwLock::new(ReprTree::new(dst_type)))
        };

        new_obj
            .repr
            .write()
            .unwrap()
            .insert_branch(old_obj.type_tag, old_obj.repr);

        self.objects.insert(name.to_string(), new_obj);
    }

    pub fn mono_view<'a, V: View + ?Sized + 'static>(
        &mut self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>,
    ) -> Option<OuterViewPort<V>>
    where
        V::Msg: Clone,
    {
        if let Some(p) = self.get_obj_port(name, type_ladder) {
            Some(p)
        } else {
            // todo : add repr with morphism constructor (if one exists)
            /*
            if let Some(ctor) = self.morphism_constructors.get(
                &MorphismType {
                    mode: MorphismMode::Mono,
                    src_type: old_obj.type_tag.clone(),
                    dst_type:
                }
            )
            */
            None
        }
}
    */
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
