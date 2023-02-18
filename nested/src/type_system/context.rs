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
//    pub mode: MorphismMode,
    pub src_type: Option<TypeTerm>,
    pub dst_type: TypeTerm,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct MorphismTypePattern {
    pub src_type: Option<TypeTerm>,
    pub dst_tyid: TypeID
}

impl From<MorphismType> for MorphismTypePattern {
    fn from(value: MorphismType) -> MorphismTypePattern {
        MorphismTypePattern {
            src_type: value.src_type,
            dst_tyid: match value.dst_type {
                TypeTerm::Type { id, args: _ } => id,
                _ => unreachable!()
            }
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Context {
    /// assigns a name to every type
    type_dict: Arc<RwLock<TypeDict>>,

    /// vertices of the graph
    nodes: HashMap< String, NestedNode >,

    /// todo: beautify
    /// types that can be edited as lists
    list_types: Vec< TypeID >,

    /// graph constructors
    morphisms: HashMap<
                   MorphismTypePattern,
                   Arc<
                           dyn Fn( NestedNode, TypeTerm ) -> Option<NestedNode>
                           + Send + Sync
                   >
               >,

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
            morphisms: HashMap::new(),
            nodes: HashMap::new(),
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

    pub fn depth(&self) -> usize {
        if let Some(parent) = self.parent.as_ref() {
            parent.read().unwrap().depth() + 1
        } else {
            0
        }
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

    pub fn add_node_ctor(&mut self, tn: &str, mk_editor: Arc<dyn Fn(Arc<RwLock<Self>>, TypeTerm, usize) -> Option<NestedNode> + Send + Sync>) {
        let dict = self.type_dict.clone();
        let mut dict = dict.write().unwrap();
        let tyid =
            if let Some(tyid) = dict.get_typeid(&tn.into()) {
                tyid
            } else {
                dict.add_typename(tn.into())
            };

        let morphism_pattern = MorphismTypePattern {
            src_type: None,
            dst_tyid: tyid
        };

        self.add_morphism(morphism_pattern, Arc::new(move |node, dst_type| {
            let ctx = node.ctx.clone().unwrap();
            let depth = node.depth;
            mk_editor(ctx, dst_type, depth)
        }));
    }

    pub fn add_morphism(
        &mut self,
        morph_type_pattern: MorphismTypePattern,
        morph_fn: Arc<
                     dyn Fn( NestedNode, TypeTerm ) -> Option<NestedNode>
                     + Send + Sync
                  >
    ) {
        self.morphisms.insert(morph_type_pattern, morph_fn);
    }

    pub fn get_morphism(&self, ty: MorphismType) -> Option<Arc<dyn Fn(NestedNode, TypeTerm) -> Option<NestedNode> + Send + Sync>> {
        let pattern = MorphismTypePattern::from(ty.clone());
        if let Some(morphism) = self.morphisms.get( &pattern ) {
            Some(morphism.clone())
        } else {
            self.parent.as_ref()?
                .read().unwrap()
                .get_morphism(ty)
        }
    }

    pub fn make_node(ctx: &Arc<RwLock<Self>>, type_term: TypeTerm, depth: usize) -> Option<NestedNode> {
        let mk_node = ctx.read().unwrap().get_morphism(MorphismType {
            src_type: None,
            dst_type: type_term.clone()
        })?;

        mk_node(NestedNode::new(depth).set_ctx(ctx.clone()), type_term)
    }

    pub fn morph_node(mut node: NestedNode, dst_type: TypeTerm) -> NestedNode {
        let ctx = node.ctx.clone().unwrap();
        let mut src_type = None;

        if let Some(data) = node.data.clone() {
            src_type = Some(data.read().unwrap().get_type().clone());
            node = node.set_data(
                ReprTree::ascend(
                    &data,
                    dst_type.clone()
                )
            );
        }

        let pattern = MorphismType { src_type, dst_type: dst_type.clone() }.into();
        let ctx = ctx.read().unwrap();
        if let Some(transform) = ctx.get_morphism(pattern) {
            if let Some(new_node) = transform(node.clone(), dst_type) {
                new_node
            } else {
                node.clone()
            }
        } else {
            node
        }
    }

    /// adds an object without any representations
    pub fn add_obj(ctx: Arc<RwLock<Context>>, name: String, typename: &str) {
        let type_tag = ctx.read().unwrap()
            .type_dict.read().unwrap()
            .type_term_from_str(typename).unwrap();

        if let Some(node) = Context::make_node(&ctx, type_tag, 0) {
            ctx.write().unwrap().nodes.insert(name, node);
        }
    }

    pub fn get_obj(&self, name: &String) -> Option<NestedNode> {
        if let Some(obj) = self.nodes.get(name) {
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
