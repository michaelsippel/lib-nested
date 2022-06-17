use {
    crate::{
        core::{InnerViewPort, OuterViewPort, Observer, ObserverBroadcast, View, ViewPort},
        vec::VecDiff,
    },
    std::sync::RwLock,
    std::{
        ops::{Deref, DerefMut},
        sync::Arc,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> View for Vec<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = VecDiff<T>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct VecBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    data: Arc<RwLock<Vec<T>>>,
    port: InnerViewPort<RwLock<Vec<T>>>
}

impl<T> VecBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn with_data(data: Vec<T>, port: InnerViewPort<RwLock<Vec<T>>>) -> Self {
        let data = Arc::new(RwLock::new(data));
        port.set_view(Some(data.clone()));

        for x in data.read().unwrap().iter().cloned() {
            port.notify(&VecDiff::Push(x));
        }
            
        VecBuffer {
            data,
            port
        }
    }

    pub fn with_port(port: InnerViewPort<RwLock<Vec<T>>>) -> Self {
        VecBuffer::with_data(vec![], port)
    }

    pub fn new() -> Self {
        VecBuffer::with_port(ViewPort::new().into_inner())
    }
    
    pub fn get_port(&self) -> OuterViewPort<RwLock<Vec<T>>> {
        self.port.0.outer()
    }

    pub fn apply_diff(&mut self, diff: VecDiff<T>) {
        let mut data = self.data.write().unwrap();
        match &diff {
            VecDiff::Clear => {
                data.clear();
            }
            VecDiff::Push(val) => {
                data.push(val.clone());
            }
            VecDiff::Remove(idx) => {
                data.remove(*idx);
            }
            VecDiff::Insert { idx, val } => {
                data.insert(*idx, val.clone());
            }
            VecDiff::Update { idx, val } => {
                data[*idx] = val.clone();
            }
        }
        drop(data);

        self.port.notify(&diff);
    }

    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    pub fn get(&self, idx: usize) -> T {
        self.data.read().unwrap()[idx].clone()
    }

    pub fn clear(&mut self) {
        self.apply_diff(VecDiff::Clear);
    }

    pub fn push(&mut self, val: T) {
        self.apply_diff(VecDiff::Push(val));
    }

    pub fn remove(&mut self, idx: usize) {
        self.apply_diff(VecDiff::Remove(idx));
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Insert { idx, val });
    }

    pub fn update(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Update { idx, val });
    }

    pub fn get_mut(&mut self, idx: usize) -> MutableVecAccess<T> {
        MutableVecAccess {
            buf: self.clone(),
            idx,
            val: self.get(idx),
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    buf: VecBuffer<T>,
    idx: usize,
    val: T,
}

impl<T> Deref for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for MutableVecAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.buf.update(self.idx, self.val.clone());
    }
}
