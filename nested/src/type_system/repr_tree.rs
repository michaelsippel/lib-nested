use {
    r3vi::view::{AnyOuterViewPort, OuterViewPort, View},
    crate::{
        type_system::{TypeTerm, Context}
    },
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct ReprTree {
    type_tag: TypeTerm,
    port: Option<AnyOuterViewPort>,
    branches: HashMap<TypeTerm, Arc<RwLock<ReprTree>>>,
}

impl std::fmt::Debug for ReprTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type: {:?}", self.type_tag)?;

        for (_k,x) in self.branches.iter() {
            write!(f, "child: {:?}", x)?;
        }

        Ok(())
    }
}

impl ReprTree {
    pub fn new(type_tag: impl Into<TypeTerm>) -> Self {
        ReprTree {
            type_tag: type_tag.into(),
            port: None,
            branches: HashMap::new(),
        }
    }

    pub fn new_arc(type_tag: impl Into<TypeTerm>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(type_tag)))
    }

    pub fn get_type(&self) -> &TypeTerm {
        &self.type_tag
    }

    pub fn from_char(ctx: &Arc<RwLock<Context>>, c: char) -> Arc<RwLock<Self>> {
        let buf = r3vi::buffer::singleton::SingletonBuffer::<char>::new(c);
        ReprTree::new_leaf(
            (ctx, "( Char )"),
            buf.get_port().into()
        )
    }

    pub fn new_leaf(type_tag: impl Into<TypeTerm>, port: AnyOuterViewPort) -> Arc<RwLock<Self>> {
        let mut tree = ReprTree::new(type_tag.into());
        tree.insert_leaf(vec![].into_iter(), port);
        Arc::new(RwLock::new(tree))
    }

    pub fn insert_branch(&mut self, repr: Arc<RwLock<ReprTree>>) {
        self.branches.insert(repr.clone().read().unwrap().type_tag.clone(), repr.clone());
    }

    pub fn insert_leaf(
        &mut self,
        mut type_ladder: impl Iterator<Item = TypeTerm>,
        port: AnyOuterViewPort,
    ) {
        if let Some(type_term) = type_ladder.next() {
            if let Some(next_repr) = self.branches.get(&type_term) {
                next_repr.write().unwrap().insert_leaf(type_ladder, port);
            } else {
                let mut next_repr = ReprTree::new(type_term.clone());
                next_repr.insert_leaf(type_ladder, port);
                self.insert_branch(Arc::new(RwLock::new(next_repr)));
            }
        } else {
            self.port = Some(port);
        }
    }

    //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

    pub fn get_port<V: View + ?Sized + 'static>(&self) -> Option<OuterViewPort<V>>
    where
        V::Msg: Clone,
    {
        Some(
            self.port
                .clone()?
                .downcast::<V>()
                .ok()?
        )
    }

    pub fn get_view<V: View + ?Sized + 'static>(&self) -> Option<Arc<V>>
    where
        V::Msg: Clone,
    {
            self.get_port::<V>()?
                .get_view()
    }

    pub fn descend(&self, dst_type: impl Into<TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        self.branches.get(&dst_type.into()).cloned()
    }

    pub fn descend_ladder(rt: &Arc<RwLock<Self>>, mut repr_ladder: impl Iterator<Item = TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        if let Some(first) = repr_ladder.next() {
            let rt = rt.read().unwrap();
            repr_ladder.fold(
                rt.descend(first),
                |s, t| s?.read().unwrap().descend(t))
        } else {
            Some(rt.clone())
        }
    }

    pub fn ascend(rt: &Arc<RwLock<Self>>, type_term: impl Into<TypeTerm>) -> Arc<RwLock<ReprTree>> {
        let mut n = Self::new(type_term);
        n.insert_branch(rt.clone());
        Arc::new(RwLock::new(n))
    }

/*
    pub fn add_iso_repr(
        &self,
        type_ladder: impl Iterator<Item = TypeTerm>,
        morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>,
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
                    if let Some(ctor) = morphism_constructors.get(&MorphismType {
                        mode: MorphismMode::Iso,
                        src_type: src_type.clone(),
                        dst_type: dst_type.clone(),
                    }) {
                        let new_obj = ctor(Object {
                            type_tag: src_type.clone(),
                            repr: cur_repr
                                .read()
                                .unwrap()
                                .branches
                                .get(&src_type)
                                .unwrap()
                                .clone(),
                        });

                        assert!(new_obj.type_tag == dst_type);

                        obj = Some(new_obj);
                        break;
                    }
                }

                if let Some(obj) = obj {
                    cur_repr
                        .write()
                        .unwrap()
                        .insert_branch(obj.type_tag, obj.repr);
                } else {
                    panic!("could not find matching isomorphism!");
                }
            }
        }
    }

    pub fn add_mono_repr<'a>(
        &self,
        type_ladder: impl Iterator<Item = TypeTerm>,
        morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>,
    ) {
        let mut cur_type = self.type_tag.clone();
        let mut cur_repr = self.repr.clone();   

        for dst_type in type_ladder {
            if let Some(next_repr) = self.repr.read().unwrap().branches.get(&dst_type) {
                // go deeper
                cur_type = dst_type;
                cur_repr = next_repr.clone();
            } else {
                if let Some(constructor) = morphism_constructors.get(&MorphismType {
                    mode: MorphismMode::Mono,
                    src_type: cur_type.clone(),
                    dst_type: dst_type.clone(),
                }) {
                    let new_obj = constructor(Object {
                        type_tag: cur_type.clone(),
                        repr: cur_repr
                            .read()
                            .unwrap()
                            .branches
                            .get(&cur_type)
                            .unwrap()
                            .clone(),
                    });

                    assert!(new_obj.type_tag == dst_type);
                    cur_repr
                        .write()
                        .unwrap()
                        .insert_branch(new_obj.type_tag.clone(), new_obj.repr.clone());

                    cur_type = new_obj.type_tag;
                    cur_repr = new_obj.repr;
                }
            }
        }
    }

    // replace with higher-level type in which self is a repr branch
    pub fn epi_cast<'a>(
        &self,
        _type_ladder: impl Iterator<Item = TypeTerm>,
        _morphism_constructors: &HashMap<MorphismType, Box<dyn Fn(Object) -> Object>>,
    ) {
        // todo        
}
    */
}

