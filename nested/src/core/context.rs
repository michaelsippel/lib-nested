use {
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        any::Any
    },
    crate::{
        bimap::Bimap,
        core::type_term::{TypeID, TypeTerm}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub enum ReprTree {
    Leaf(Arc<dyn Any + Send + Sync>),
    Branch(HashMap<TypeTerm, Arc<RwLock<ReprTree>>>)
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct Object {
    type_tag: TypeTerm,
    repr: Arc<RwLock<ReprTree>>
}

impl Object {
    fn downcast(&self, repr_type: TypeTerm) -> Option<Object> {
        match &*self.repr.read().unwrap() {
            ReprTree::Leaf(data) =>
                if self.type_tag == repr_type {
                    Some(self.clone())
                } else {
                    None
                },
            ReprTree::Branch(reprs) =>
                Some(Object{
                    type_tag: repr_type.clone(),
                    repr: reprs.get(&repr_type)?.clone()
                })
        }
    }

    fn downcast_chain<'a>(
        &self,
        repr_chain: impl Iterator<Item = &'a TypeTerm>
    ) -> Option<Object> {
        repr_chain.fold(
            Some(self.clone()),
            |s, t| s?.downcast(t.clone())
        )
    }

    fn get_data<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        match &*self.repr.read().unwrap() {
            ReprTree::Leaf(data) => Arc::downcast::<T>(data.clone()).ok(),
            _ => None
        }
    }    
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TypeDict {
    typenames: Bimap::<String, u64>,
    type_id_counter: u64
}

impl TypeDict {
    pub fn new() -> Self {
        TypeDict {
            typenames: Bimap::new(),
            type_id_counter: 0
        }
    }

    pub fn add_typename(&mut self, tn: String) {
        self.typenames.insert(tn, self.type_id_counter);
        self.type_id_counter += 1;
    }

    pub fn type_term_from_str(&self, typename: &str) -> Option<TypeTerm> {
        TypeTerm::from_str(typename, &self.typenames.mλ)
    }

    pub fn type_term_to_str(&self, term: &TypeTerm) -> String {
        term.to_str(&self.typenames.my)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Context {
    type_dict: TypeDict,

    // map type_id -> constructor
    constructors: HashMap<u64, Box<Fn() -> Arc<dyn Any + Send + Sync>>>,

    objects: HashMap<String, Object>
}

impl Context {
    pub fn add_obj(
        &mut self,
        name: String,
        typename: &str
    ) {
        self.objects.insert(
            name,
            Object {
                type_tag: self.type_dict.type_term_from_str(typename).unwrap(),
                repr: Arc::new(RwLock::new(ReprTree::Branch(HashMap::new())))
            }
        );
    }

    pub fn add_repr(&mut self, name: &String, typename: &str, repr: Arc<RwLock<ReprTree>>) {
        match &mut *self.objects.get_mut(name).unwrap().repr.write().unwrap() {
            ReprTree::Leaf(_) => {/*error*/},
            ReprTree::Branch(repr_map) => {
                repr_map.insert(self.type_dict.type_term_from_str(typename).unwrap(), repr.clone());
            }
        }
    }

    pub fn get_obj(&self, name: &String) -> Option<Object> {
        self.objects.get(name).cloned()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

