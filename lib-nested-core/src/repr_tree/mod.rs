pub mod context;
pub mod morphism;

#[cfg(test)]
mod tests;

pub use {
    context::{Context},
    morphism::{MorphismType, GenericReprTreeMorphism, MorphismBase}
};

use {
    r3vi::{
        view::{
            ViewPort, OuterViewPort,
            AnyViewPort, AnyInnerViewPort, AnyOuterViewPort,
            port::UpdateTask,
            View,
            singleton::*,
            sequence::*,
            list::*
        },
        buffer::{singleton::*, vec::*}
    },
    laddertypes::{TypeTerm},
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
        any::Any
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct ReprLeaf {
    out_port: AnyViewPort,
    in_port: AnyInnerViewPort,
    data: Option< Arc<dyn Any + Send + Sync> >,

    /// keepalive for the observer that updates the buffer from in_port
    keepalive: Option<Arc<dyn Any + Send + Sync>>,
}

#[derive(Clone)]
pub struct ReprTree {
    type_tag: TypeTerm,
    branches: HashMap<TypeTerm, Arc<RwLock<ReprTree>>>,
    leaf: Option< ReprLeaf >
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl std::fmt::Debug for ReprTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "| type: {:?}", self.type_tag)?;

        for (_k,x) in self.branches.iter() {
            writeln!(f, "|--> child: {:?}", x)?;
        }

        Ok(())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl ReprLeaf {
    pub fn from_view<V>( src_port: OuterViewPort<V> ) -> Self
    where V: View + ?Sized + 'static,
        V::Msg: Clone
    {
        let mut in_port = ViewPort::<V>::new();
        in_port.attach_to(src_port);

        let mut buf_port = ViewPort::<V>::new();
        buf_port.attach_to(in_port.outer());

        ReprLeaf {
            keepalive: None,
            in_port: in_port.inner().into(),
            out_port: buf_port.into(),
            data: None, 
        }
    }

    pub fn attach_to<V>(&mut self, src_port: OuterViewPort<V>)
    where V: View + ?Sized + 'static,
        V::Msg: Clone
    {
        self.in_port.clone()
            .downcast::<V>().ok().unwrap()
            .0.attach_to( src_port );
    }

    pub fn from_singleton_buffer<T>( buffer: SingletonBuffer<T> ) -> Self
    where T: Clone + Send + Sync + 'static
    {
        let in_port = ViewPort::<dyn SingletonView<Item = T>>::new();
        ReprLeaf {
            keepalive: Some(buffer.attach_to(in_port.outer())),
            in_port: in_port.inner().into(),
            out_port: buffer.get_port().0.into(),
            data: Some(buffer.into_inner())
        }
    }

    pub fn from_vec_buffer<T>( buffer: VecBuffer<T> ) -> Self
    where T: Clone + Send + Sync + 'static
    {
        eprintln!("ReprLeaf from vec buffer (LEN ={})", buffer.len());
        let in_port = ViewPort::< dyn ListView<T> >::new();
        ReprLeaf {
            keepalive: Some(buffer.attach_to(in_port.outer())),
            in_port: in_port.inner().into(),
            out_port: buffer.get_port().0.into(),
            data: Some(buffer.into_inner())
        }
    }

    pub fn as_singleton_buffer<T>(&mut self) -> Option<SingletonBuffer<T>>
    where T: Clone + Send + Sync + 'static
    {
        let sgl_port = self.get_port::< dyn SingletonView<Item = T> >().unwrap().0;

        let data_arc =
            if let Some(data) = self.data.as_ref() {
                data.clone().downcast::<RwLock<T>>().ok()
            } else {
                sgl_port.update();
                let value = sgl_port.outer().get_view().unwrap().get();
                eprintln!("make new data ARC from old value");
                Some(Arc::new(RwLock::new( value )))
            };

        if let Some(data_arc) = data_arc {
            self.data = Some(data_arc.clone() as Arc<dyn Any + Send + Sync>);
            let buf = SingletonBuffer {
                value: data_arc,
                port: sgl_port.inner()
            };
            self.keepalive = Some(buf.attach_to(
                self.in_port.0.clone()
                    .downcast::<dyn SingletonView<Item = T>>()
                    .ok().unwrap()
                    .outer()
            ));
            Some(buf)
        } else {
            None
        }
    }

    pub fn as_vec_buffer<T>(&mut self) -> Option<VecBuffer<T>>
    where T: Clone + Send + Sync + 'static
    {
        let vec_port = self.get_port::< RwLock<Vec<T>> >().unwrap().0;

        let data_arc =
            if let Some(data) = self.data.as_ref() {
                eprintln!("downcast existing vec-data");
                data.clone().downcast::<RwLock<Vec<T>>>().ok()
            } else {
                vec_port.update();
                if let Some(value) = vec_port.outer().get_view() {
                    let value = value.read().unwrap().clone();
                    eprintln!("make new data ARC from old VECTOR-value");
                    Some(Arc::new(RwLock::new( value )))
                } else {
                    eprintln!("no data vec");
                    Some(Arc::new(RwLock::new( Vec::new() )))
//                    None
                }
            };

        if let Some(data_arc) = data_arc {
            eprintln!("ReprLeaf: have Vec-like data-arc");
            eprintln!("LEN = {}", data_arc.read().unwrap().len());

            self.data = Some(data_arc.clone() as Arc<dyn Any + Send + Sync>);
            let buf = VecBuffer {
                data: data_arc,
                port: vec_port.inner()
            };
            self.keepalive = Some(buf.attach_to(
                self.in_port.0.clone()
                    .downcast::< dyn ListView<T> >()
                    .ok().unwrap()
                    .outer()
            ));
            Some(buf)
        } else {
            None
        }
    }


    pub fn get_port<V>(&self) -> Option<OuterViewPort<V>>
    where V: View + ?Sized + 'static,
        V::Msg: Clone
    {
        self.out_port.clone().downcast::<V>().ok().map(|p| p.outer())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl ReprTree {
    pub fn new(type_tag: impl Into<TypeTerm>) -> Self {
        ReprTree {
            type_tag: type_tag.into(),
            branches: HashMap::new(),
            leaf: None
        }
    }

    pub fn new_arc(type_tag: impl Into<TypeTerm>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(type_tag)))
    }

    pub fn get_type(&self) -> &TypeTerm {
        &self.type_tag
    }

    pub fn insert_branch(&mut self, repr: Arc<RwLock<ReprTree>>) {
        self.branches.insert(repr.clone().read().unwrap().type_tag.clone(), repr.clone());
    }

    pub fn from_char(ctx: &Arc<RwLock<Context>>, c: char ) -> Arc<RwLock<Self>> {
        ReprTree::from_singleton_buffer(
            Context::parse(ctx, "Char"),
            SingletonBuffer::new(c)
        )
    }

    pub fn from_view<V>( type_tag: impl Into<TypeTerm>, view: OuterViewPort<V> ) -> Arc<RwLock<Self>>
    where V: View + ?Sized + 'static,
        V::Msg: Clone
    {
        let mut rt = ReprTree::new(type_tag);
        rt.leaf = Some(ReprLeaf::from_view(view));
        Arc::new(RwLock::new(rt))
    }

    pub fn from_singleton_buffer<T>( type_tag: impl Into<TypeTerm>, buf: SingletonBuffer<T> ) -> Arc<RwLock<Self>>
    where T: Clone + Send + Sync + 'static
    {
        let mut rt = ReprTree::new(type_tag);
        rt.leaf = Some(ReprLeaf::from_singleton_buffer(buf));
        Arc::new(RwLock::new(rt))
    }


    pub fn from_vec_buffer<T>( type_tag: impl Into<TypeTerm>, buf: VecBuffer<T> ) -> Arc<RwLock<Self>>
    where T: Clone + Send + Sync + 'static
    {
        let mut rt = ReprTree::new(type_tag);
        rt.leaf = Some(ReprLeaf::from_vec_buffer(buf));
        Arc::new(RwLock::new(rt))
    }

    /// find, and if necessary, create corresponding path in repr-tree.
    /// Attach src_port to input of that node
    pub fn attach_leaf_to<V>(
        &mut self,
        mut type_ladder: impl Iterator<Item = TypeTerm>,
        src_port: OuterViewPort<V>
    )
    where V: View + ?Sized + 'static,
        V::Msg: Clone
    {
        if let Some(rung_type) = type_ladder.next() {
            if let Some(next_repr) = self.branches.get(&rung_type) {
                next_repr.write().unwrap().attach_leaf_to(type_ladder, src_port);
            } else {
                let mut next_repr = ReprTree::new(rung_type.clone());
                next_repr.attach_leaf_to(type_ladder, src_port);
                self.insert_branch(Arc::new(RwLock::new(next_repr)));
            }
        } else {
            if let Some(leaf) = self.leaf.as_mut() {
                leaf.attach_to(src_port);
            } else {
                self.leaf = Some(ReprLeaf::from_view(src_port));
            }
        }
    }

    pub fn insert_leaf(
        &mut self,
        mut type_ladder: impl Iterator<Item = TypeTerm>,
        leaf: ReprLeaf
    ) {
        if let Some(type_term) = type_ladder.next() {
            if let Some(next_repr) = self.branches.get(&type_term) {
                next_repr.write().unwrap().insert_leaf(type_ladder, leaf);
            } else {
                let mut next_repr = ReprTree::new(type_term.clone());
                next_repr.insert_leaf(type_ladder, leaf);
                self.insert_branch(Arc::new(RwLock::new(next_repr)));
            }
        } else {
            self.leaf = Some(leaf)
        }
    }

    //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

    pub fn descend_one(&self, dst_type: impl Into<TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        let dst_type = dst_type.into();
        assert!( dst_type.is_flat() );
        self.branches.get(&dst_type).cloned()
    }

    pub fn descend_ladder(rt: &Arc<RwLock<Self>>, mut repr_ladder: impl Iterator<Item = TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        if let Some(first) = repr_ladder.next() {
            let rt = rt.read().unwrap();
            repr_ladder.fold(
                rt.descend_one(first),
                |s, t| s?.descend(t))
        } else {
            Some(rt.clone())
        }
    }

    pub fn descend(rt: &Arc<RwLock<Self>>, dst_type: impl Into<TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        ReprTree::descend_ladder(rt, dst_type.into().get_lnf_vec().into_iter())
    }

    pub fn ascend(rt: &Arc<RwLock<Self>>, type_term: impl Into<TypeTerm>) -> Arc<RwLock<ReprTree>> {
        let mut n = Self::new(type_term);
        n.insert_branch(rt.clone());
        Arc::new(RwLock::new(n))
    }

    //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

    pub fn singleton_buffer<T: Clone + Send + Sync + 'static>(&mut self) -> Option<SingletonBuffer<T>> {
        if let Some(leaf) = self.leaf.as_mut() {
            leaf.as_singleton_buffer::<T>()
        } else {
            // create new singleton buffer
            /*
            // default value??
            let buf = SingletonBuffer::<T>::default();
            self.leaf = Some(ReprLeaf::from_singleton_buffer(buf.clone()));
            Some(buf)
            */
            None
        }
    }

    pub fn vec_buffer<T: Clone + Send + Sync + 'static>(&mut self) -> Option<VecBuffer<T>> {
        if let Some(leaf) = self.leaf.as_mut() {
            leaf.as_vec_buffer::<T>()
        } else {
            None
        }
    }

    //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

    pub fn get_port<V: View + ?Sized + 'static>(&self) -> Option<OuterViewPort<V>>
    where
        V::Msg: Clone,
    {
        if let Some(leaf) = self.leaf.as_ref() {
            leaf.get_port::<V>()
        } else {
            None
        }
    }

    pub fn get_view<V: View + ?Sized + 'static>(&self) -> Option<Arc<V>>
    where
        V::Msg: Clone,
    {
        self.get_port::<V>()?
            .get_view()
    }

    //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

    pub fn view_seq<T: 'static>(&self) -> OuterViewPort<dyn SequenceView<Item = T>> {
        self.get_port::<dyn SequenceView<Item = T>>().expect("no sequence-view available")
    }

    pub fn view_char(&self) -> OuterViewPort<dyn SingletonView<Item = char>> {
        self.get_port::<dyn SingletonView<Item = char>>().expect("no char-view available")
    }

    pub fn view_u8(&self) -> OuterViewPort<dyn SingletonView<Item = u8>> {
        self.get_port::<dyn SingletonView<Item = u8>>().expect("no u8-view available")
    }

    pub fn view_u64(&self) -> OuterViewPort<dyn SingletonView<Item = u64>> {
        self.get_port::<dyn SingletonView<Item = u64>>().expect("no u64-view available")
    }
}



//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ReprTreeExt {
    fn get_type(&self) -> TypeTerm;

    fn insert_leaf(&mut self, type_ladder: impl Into<TypeTerm>, leaf: ReprLeaf);
    fn insert_branch(&mut self, repr: Arc<RwLock<ReprTree>>);
    fn descend(&self, target_type: impl Into<TypeTerm>) -> Option<Arc<RwLock<ReprTree>>>;

    fn view_char(&self) -> OuterViewPort<dyn SingletonView<Item = char>>;
    fn view_u8(&self) -> OuterViewPort<dyn SingletonView<Item = u8>>;
    fn view_u64(&self) -> OuterViewPort<dyn SingletonView<Item = u64>>;

    fn singleton_buffer<T: Clone + Send + Sync + 'static>(&self) -> SingletonBuffer<T>;
    fn vec_buffer<T: Clone + Send + Sync + 'static>(&self) -> VecBuffer<T>;
}

impl ReprTreeExt for Arc<RwLock<ReprTree>> {
    fn get_type(&self) -> TypeTerm {
        self.read().unwrap().get_type().clone()
    }

    fn insert_leaf(&mut self, type_ladder: impl Into<TypeTerm>, leaf: ReprLeaf) {
        self.write().unwrap().insert_leaf(type_ladder.into().get_lnf_vec().into_iter(), leaf)
    }

    fn insert_branch(&mut self, repr: Arc<RwLock<ReprTree>>) {
        self.write().unwrap().insert_branch(repr)
    }

    fn descend(&self, target_type: impl Into<TypeTerm>) -> Option<Arc<RwLock<ReprTree>>> {
        ReprTree::descend( self, target_type )
    }

    fn view_char(&self) -> OuterViewPort<dyn SingletonView<Item = char>> {
        self.read().unwrap().view_char()
    }

    fn view_u8(&self) -> OuterViewPort<dyn SingletonView<Item = u8>> {
        self.read().unwrap().view_u8()
    }

    fn view_u64(&self) -> OuterViewPort<dyn SingletonView<Item = u64>> {
        self.read().unwrap().view_u64()
    }

    fn singleton_buffer<T: Clone + Send + Sync + 'static>(&self) -> SingletonBuffer<T> {
        self.write().unwrap().singleton_buffer::<T>().expect("")
    }

    fn vec_buffer<T: Clone + Send + Sync + 'static>(&self) -> VecBuffer<T> {
        self.write().unwrap().vec_buffer::<T>().expect("")
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

