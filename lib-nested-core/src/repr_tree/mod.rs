pub mod context;
pub mod morphism;

pub use {
    context::{Context},
    morphism::{MorphismType, GenericReprTreeMorphism, MorphismBase}
};

use {
    r3vi::view::{AnyOuterViewPort, OuterViewPort, View},
    laddertypes::{TypeTerm},
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
            Context::parse(ctx, "Char"),
            buf.get_port().into()
        )
    }

    pub fn from_u64(ctx: &Arc<RwLock<Context>>, v: u64) -> Arc<RwLock<Self>> {
        let buf = r3vi::buffer::singleton::SingletonBuffer::<u64>::new(v);
        ReprTree::new_leaf(
            Context::parse(ctx, "<MachineInt 64>"),
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
}

