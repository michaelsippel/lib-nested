use {
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        any::Any
    },
    crate::{
        core::{
            type_term::{
                TypeID,
                TypeTerm,
                TypeDict
            },
            View,
            OuterViewPort,
            AnyOuterViewPort
        }
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct ReprTree {
    port: Option<AnyOuterViewPort>,
    branches: HashMap<TypeTerm, Arc<RwLock<ReprTree>>>
}

impl ReprTree {
    pub fn new() -> Self {
        ReprTree {
            port: None,
            branches: HashMap::new()
        }
    }

    pub fn new_leaf(
        port: AnyOuterViewPort
    ) -> Arc<RwLock<Self>> {
        let mut tree = ReprTree::new();
        tree.insert_leaf(vec![].into_iter(), port);
        Arc::new(RwLock::new(tree))
    }

    pub fn insert_branch(
        &mut self,
        type_tag: TypeTerm,
        repr: Arc<RwLock<ReprTree>>
    ) {
        self.branches.insert(type_tag, repr);
    }

    pub fn insert_leaf(
        &mut self,
        mut type_ladder: impl Iterator<Item = TypeTerm>,
        port: AnyOuterViewPort
    ) {
        if let Some(type_term) = type_ladder.next() {
            if let Some(next_repr) = self.branches.get(&type_term) {
                next_repr.write().unwrap().insert_leaf(type_ladder, port);
            } else {
                let mut next_repr = ReprTree::new();
                next_repr.insert_leaf(type_ladder, port);
                self.insert_branch(type_term, Arc::new(RwLock::new(next_repr)));
            }
        } else {
            self.port = Some(port);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct Object {
    pub type_tag: TypeTerm,
    pub repr: Arc<RwLock<ReprTree>>
}

impl Object {
    pub fn get_port<V: View + ?Sized + 'static>(&self) -> Option<OuterViewPort<V>> {
        Some(self.repr.read().unwrap().port.clone()?.downcast::<V>().unwrap())
    }

    pub fn downcast(&self, dst_type: TypeTerm) -> Option<Object> {
        if let Some(repr) = self.repr.read().unwrap().branches.get(&dst_type) {
            Some(Object {
                type_tag: dst_type,
                repr: repr.clone()
            })
        } else {
            None
        }
    }

    fn downcast_ladder(
        &self,
        repr_ladder: impl Iterator<Item = TypeTerm>
    ) -> Option<Object> {
        repr_ladder.fold(
            Some(self.clone()),
            |s, t| s?.downcast(t.clone())
        )
    }

    pub fn add_iso_repr(
        &self,
        type_ladder: impl Iterator<Item = TypeTerm>,
        morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>
    ) {
        let mut cur_repr = self.repr.clone();

        for dst_type in type_ladder {
            if let Some(next_repr) = self.repr.read().unwrap().branches.get(&dst_type) {
                // go deeper
                cur_repr = next_repr.clone();
            } else {
                // search for morphism constructor and insert new repr
                let mut obj = None;

                for src_type in cur_repr.read().unwrap().branches.keys() {
                    if let Some(ctor) = morphism_constructors.get(
                        &MorphismType {
                            mode: MorphismMode::Iso,
                            src_type: src_type.clone(),
                            dst_type: dst_type.clone()
                        }
                    ) {
                        let new_obj = ctor(
                            Object {
                                type_tag: src_type.clone(),
                                repr: cur_repr.read().unwrap()
                                    .branches
                                    .get(&src_type).unwrap().clone()
                            }
                        );

                        assert!(new_obj.type_tag == dst_type);

                        obj = Some(new_obj);
                        break;
                    }
                }

                if let Some(obj) = obj {
                    cur_repr.write().unwrap().insert_branch(obj.type_tag, obj.repr);
                } else {
                    panic!("could not find matching isomorphism!");
                }
            }
        }
    }

    pub fn add_mono_repr<'a>(
        &self,
        type_ladder: impl Iterator<Item = TypeTerm>,
        morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>
    ) {
        let mut cur_type = self.type_tag.clone();
        let mut cur_repr = self.repr.clone();

        for dst_type in type_ladder {
            if let Some(next_repr) = self.repr.read().unwrap().branches.get(&dst_type) {
                // go deeper
                cur_type = dst_type;
                cur_repr = next_repr.clone();
            } else {
                if let Some(constructor) = morphism_constructors.get(
                    &MorphismType {
                        mode: MorphismMode::Mono,
                        src_type: cur_type.clone(),
                        dst_type: dst_type.clone()
                    }
                ) {
                    let new_obj = constructor(
                        Object {
                            type_tag: cur_type.clone(),
                            repr: cur_repr.read().unwrap()
                                .branches
                                .get(&cur_type).unwrap().clone()
                        }
                    );

                    assert!(new_obj.type_tag == dst_type);
                    cur_repr.write().unwrap().insert_branch(new_obj.type_tag.clone(), new_obj.repr.clone());

                    cur_type = new_obj.type_tag;
                    cur_repr = new_obj.repr;
                }
            }
        }
    }

    // replace with higher-level type in which self is a repr branch
    pub fn epi_cast<'a>(
        &self,
        type_ladder: impl Iterator<Item = TypeTerm>,
        morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>
    ) {
        // todo        
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

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
    Any
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct MorphismType {
    pub mode: MorphismMode,
    pub src_type: TypeTerm,
    pub dst_type: TypeTerm
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Context {
    type_dict: TypeDict,
    default_constructors: HashMap<TypeTerm, Box<dyn Fn() -> Object + Send + Sync>>,
    morphism_constructors: HashMap<MorphismType, Box<dyn Fn(Object) -> Object + Send + Sync>>,
    objects: HashMap<String, Object>,
    parent: Option<Arc<RwLock<Context>>>
}

impl Context {
    pub fn with_parent(parent: Option<Arc<RwLock<Context>>>) -> Self {
        Context {
            type_dict: TypeDict::new(),
            default_constructors: HashMap::new(),
            morphism_constructors: HashMap::new(),
            objects: HashMap::new(),
            parent
        }
    }

    pub fn new() -> Self {
        Context::with_parent(None)
    }

    pub fn add_typename(&mut self, tn: String) {
        self.type_dict.add_typename(tn);
    }

    pub fn type_term_from_str(&self, tn: &str) -> Option<TypeTerm> {
        self.type_dict.type_term_from_str(&tn)
    }

    pub fn add_morphism(
        &mut self,
        morph_type: MorphismType,
        morph_fn: Box<dyn Fn(Object) -> Object + Send + Sync>
    ) {
        self.morphism_constructors.insert(morph_type, morph_fn);
    }

    /// adds an object without any representations
    pub fn add_obj(
        &mut self,
        name: String,
        typename: &str
    ) {
        let type_tag = self.type_dict.type_term_from_str(typename).unwrap();

        self.objects.insert(
            name,
            if let Some(ctor) = self.default_constructors.get(&type_tag) {
                ctor()
            } else {
                Object {
                    type_tag,
                    repr: Arc::new(RwLock::new(ReprTree::new()))
                }

            }
        );
    }

    pub fn get_obj(
        &self,
        name: &String
    ) -> Option<Object> {
        if let Some(obj) = self.objects.get(name) {
            Some(obj.clone())
        } else if let Some(parent) = self.parent.as_ref() {
            parent.read().unwrap().get_obj(name)
        } else {
            None
        }
    }
 
    pub fn get_obj_port<
        'a,
        V: View + ?Sized + 'static
    >(
        &self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>
    ) -> Option<OuterViewPort<V>> {
        self.get_obj(&name.into())?
            .downcast_ladder(
                type_ladder.map(|tn| self.type_dict.type_term_from_str(tn).unwrap())
            )?
            .get_port()
    }

    pub fn insert_repr<'a>(
        &mut self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>,
        port: AnyOuterViewPort
    ) {
        self.get_obj(&name.to_string()).unwrap()
            .repr.write().unwrap()
            .insert_leaf(
                type_ladder.map(|tn| self.type_dict.type_term_from_str(tn).unwrap()),
                port
            );
    }

    pub fn epi_cast(
        &mut self,
        name: &str,
        typename: &str
    ) {
        let dst_type = self.type_dict.type_term_from_str(typename).unwrap();
        let mut old_obj = self.objects.get(&name.to_string()).unwrap().clone();
        let mut new_obj =
            if let Some(ctor) = self.morphism_constructors.get(
                &MorphismType {
                    mode: MorphismMode::Epi,
                    src_type: old_obj.type_tag.clone(),
                    dst_type: dst_type.clone()
                }
            ) {
                ctor(old_obj.clone())
            } else {
                Object {
                    type_tag: dst_type,
                    repr: Arc::new(RwLock::new(ReprTree::new()))
                }
            };

        new_obj.repr.write().unwrap().insert_branch(
            old_obj.type_tag,
            old_obj.repr
        );

        self.objects.insert(name.to_string(), new_obj);
    }

    pub fn mono_view<'a, V: View + ?Sized + 'static>(
        &mut self,
        name: &str,
        type_ladder: impl Iterator<Item = &'a str>
    ) -> Option<OuterViewPort<V>> {
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

/*
    pub fn _default_repr<'a>(
        &mut self,
        name: &String,
        type_ladder: impl Iterator<Item = &'a str>
    ) -> AnyOuterViewPort {
        for (i, type_term) in type_ladder.rev().enumerate() {
            match i {
                0 => {
                    if let Some(constructor) = self.default_constructors.get(&type_term) {
                        self.add_repr()
                    } else {
                        panic!("cannot find matching default constructor!");
                    }
                }
                _n => {
                    
                }
            }
        }
    }
     */
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

