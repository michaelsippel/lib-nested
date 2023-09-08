use {
    r3vi::{view::{OuterViewPort, singleton::*}, buffer::{singleton::*}},
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

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct MorphismType {
//    pub mode: MorphismMode,
    pub src_type: Option<TypeTerm>,
    pub dst_type: TypeTerm,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct MorphismTypePattern {
    pub src_tyid: Option<TypeID>,
    pub dst_tyid: TypeID
}

impl MorphismType {
    pub fn to_str(&self, ctx: &Context) -> String {
        format!("{:?} -> {:?}",
                if let Some(t) = self.src_type.as_ref() {
                    ctx.type_term_to_str(t)
                } else {
                    "None".into()
                },
                ctx.type_term_to_str(&self.dst_type))
    }
}

impl MorphismTypePattern {
    pub fn to_str(&self, ctx: &Context) -> String {
        format!("{:?} -> {:?}",
                if let Some(t) = self.src_tyid.as_ref() {
                    ctx.type_term_to_str(&TypeTerm::TypeID(t.clone()))
                } else {
                    "None".into()
                },
                ctx.type_term_to_str(&TypeTerm::TypeID(self.dst_tyid.clone())))
    }
}

impl From<MorphismType> for MorphismTypePattern {    
    fn from(value: MorphismType) -> MorphismTypePattern {
        fn strip( x: &TypeTerm ) -> TypeID {
            match x {
                TypeTerm::TypeID(id) => id.clone(),
                TypeTerm::App(args) => strip(&args[0]),
                TypeTerm::Ladder(args) => strip(&args[0]),
                    _ => unreachable!()
            }
        }

        MorphismTypePattern {
            src_tyid: value.src_type.map(|x| strip(&x)),
            dst_tyid: strip(&value.dst_type)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct Context {
    /// assigns a name to every type
    pub type_dict: Arc<RwLock<TypeDict>>,

    /// named vertices of the graph
    nodes: HashMap< String, NestedNode >,

    /// todo: beautify
    /// types that can be edited as lists
    pub list_types: Vec< TypeID >,
    pub meta_chars: Vec< char >,

    /// graph constructors
    /// TODO: move into separate struct MorphismMap or something
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

impl Default for Context {
    fn default() -> Context {
        let mut ctx = Context::new();

        ctx.add_list_typename("Sequence");
        ctx.add_synonym("Seq", "Sequence");
        ctx.add_list_typename("SepSeq");
        ctx.add_typename("NestedNode");
        ctx.add_typename("TerminalEvent");
        
        crate::editors::list::init_ctx( &mut ctx );
        crate::editors::char::init_ctx( &mut ctx );
        crate::editors::integer::init_ctx( &mut ctx );
        crate::editors::typeterm::init_ctx( &mut ctx );

        ctx
    }
}

impl Into<TypeTerm> for (&Arc<RwLock<Context>>, &str) {
    fn into(self) -> TypeTerm {
        self.0.read().unwrap().type_term_from_str(self.1).expect("could not parse type term")
    }
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
            meta_chars: match parent.as_ref() {
                Some(p) => p.read().unwrap().meta_chars.clone(),
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

    pub fn add_typename(&mut self, tn: &str) -> TypeID {
        self.type_dict.write().unwrap().add_typename(tn.to_string())
    }

    pub fn add_varname(&mut self, vn: &str) -> TypeID {
        self.type_dict.write().unwrap().add_varname(vn.to_string())
    }

    pub fn add_synonym(&mut self, new: &str, old: &str) {
        self.type_dict.write().unwrap().add_synonym(new.to_string(), old.to_string());
    }

    pub fn add_list_typename(&mut self, tn: &str) {
        let tid = self.add_typename(tn);
        self.list_types.push( tid );
    }

    pub fn is_list_type(&self, t: &TypeTerm) -> bool {
        match t {
            TypeTerm::TypeID(id) => {
                self.list_types.contains(id)
            }
            _ => false
        }
    }

    pub fn get_typeid(&self, tn: &str) -> Option<TypeID> {
        self.type_dict.read().unwrap().get_typeid(&tn.into())
    }

    pub fn get_fun_typeid(&self, tn: &str) -> Option<u64> {
        match self.get_typeid(tn) {
            Some(TypeID::Fun(x)) => Some(x),
            _ => None
        }
    }

    pub fn get_typename(&self, tid: &TypeID) -> Option<String> {
        self.type_dict.read().unwrap().get_typename(tid)
    }

    pub fn get_var_typeid(&self, tn: &str) -> Option<u64> {
        match self.get_typeid(tn) {
            Some(TypeID::Var(x)) => Some(x),
            _ => None
        }
    }

    pub fn type_term_from_str(&self, tn: &str) -> Option<TypeTerm> {
        self.type_dict.read().unwrap().type_term_from_str(&tn)
    }

    pub fn type_term_to_str(&self, t: &TypeTerm) -> String {
        self.type_dict.read().unwrap().type_term_to_str(&t)
    }

    pub fn add_node_ctor(&mut self, tn: &str, mk_editor: Arc<dyn Fn(Arc<RwLock<Self>>, TypeTerm, OuterViewPort<dyn SingletonView<Item = usize>>) -> Option<NestedNode> + Send + Sync>) {
        let dict = self.type_dict.clone();
        let mut dict = dict.write().unwrap();

        let tyid =
            if let Some(tyid) = dict.get_typeid(&tn.into()) {
                tyid
            } else {
                dict.add_typename(tn.into())
            };

        let morphism_pattern = MorphismTypePattern {
            src_tyid: None,
            dst_tyid: tyid
        };

        drop(dict);

        self.add_morphism(morphism_pattern, Arc::new(move |node, dst_type| {
            mk_editor(node.ctx.clone(), dst_type, node.depth)
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

    pub fn make_node(ctx: &Arc<RwLock<Self>>, type_term: TypeTerm, depth: OuterViewPort<dyn SingletonView<Item = usize>>) -> Option<NestedNode> {
        let mk_node = ctx.read().unwrap().get_morphism(MorphismType {
            src_type: None,
            dst_type: type_term.clone()
        }).expect(&format!("morphism {}", ctx.read().unwrap().type_term_to_str(&type_term)));

        /* create new context per node ?? too heavy.. whats the reason? TODO */

        let new_ctx = Arc::new(RwLock::new(Context::with_parent(Some(ctx.clone()))));

        mk_node(
            NestedNode::new(new_ctx, ReprTree::new_arc(type_term.clone()), depth),
            type_term
        )
    }

    pub fn morph_node(mut node: NestedNode, dst_type: TypeTerm) -> NestedNode {
        let src_type = node.data.read().unwrap().get_type().clone();
        let pattern = MorphismType { src_type: Some(src_type), dst_type: dst_type.clone() };

        /* it is not univesally true to always use ascend.
         */
        node.data =
            ReprTree::ascend(
                &node.data,
                dst_type.clone()
            );

        let m = node.ctx.read().unwrap().get_morphism(pattern.clone());
        if let Some(transform) = m {
            if let Some(new_node) = transform(node.clone(), dst_type) {
                new_node
            } else {
                node.clone()
            }
        } else {
            eprintln!("could not find morphism {}", pattern.to_str(&node.ctx.read().unwrap()));
            node
        }
    }

    /// adds an object without any representations
    pub fn add_obj(ctx: Arc<RwLock<Context>>, name: String, typename: &str) {
        let type_tag = ctx.read().unwrap()
            .type_dict.read().unwrap()
            .type_term_from_str(typename).unwrap();

        if let Some(node) = Context::make_node(&ctx, type_tag, SingletonBuffer::new(0).get_port()) {
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

