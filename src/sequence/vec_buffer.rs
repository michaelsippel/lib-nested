use {
    std::{
        sync::{Arc, RwLock},
        ops::{Deref, DerefMut}
    },
    crate::{
        core::{View, Observer, ObserverExt, ObserverBroadcast, ViewPort, InnerViewPort, OuterViewPort},
        sequence::SequenceView,
    }
};

pub enum VecDiff<T> {
    Push(T),
    Remove(usize),
    Insert{ idx: usize, val: T },
    Update{ idx: usize, val: T }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> View for Vec<T>
where T: Clone + Send + Sync + 'static {
    type Msg = VecDiff<T>;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct VecSequence<T>
where T: Clone + Send + Sync + 'static {
    cur_len: RwLock<usize>,
    data: Option<Arc<RwLock<Vec<T>>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = T>>>>
}

impl<T> OuterViewPort<RwLock<Vec<T>>>
where T: Clone + Send + Sync + 'static {
    pub fn to_sequence(&self) -> OuterViewPort<dyn SequenceView<Item = T>> {
        let port = ViewPort::new();
        let vec_seq = VecSequence::new(port.inner());
        self.add_observer(vec_seq.clone());
        port.into_outer()
    }
}

impl<T> VecSequence<T>
where T: Clone + Send + Sync + 'static {
    pub fn new(
        port: InnerViewPort<dyn SequenceView<Item = T>>
    ) -> Arc<RwLock<Self>> {
        let seq = Arc::new(RwLock::new(
            VecSequence {
                cur_len: RwLock::new(0),
                data: None,
                cast: port.get_broadcast()
            }
        ));
        port.set_view(Some(seq.clone()));
        seq
    }
}

impl<T> Observer<RwLock<Vec<T>>> for VecSequence<T>
where T: Clone + Send + Sync + 'static {
    fn reset(&mut self, view: Option<Arc<RwLock<Vec<T>>>>) {
        let old_len = self.len().unwrap();
        self.data = view;
        let new_len = self.len().unwrap();
        self.cast.notify_each(0 .. std::cmp::max(old_len, new_len));
    }

    fn notify(&self, diff: &VecDiff<T>) {
        match diff {
            VecDiff::Push(_) => {
                let l = {
                    let mut l = self.cur_len.write().unwrap();
                    *l += 1;
                    *l
                };
                self.cast.notify(&(l - 1));
            },
            VecDiff::Remove(idx) => {
                let l = {
                    let mut l = self.cur_len.write().unwrap();
                    *l -= 1;
                    *l + 1
                };
                self.cast.notify_each(*idx .. l);
            },
            VecDiff::Insert{ idx, val: _ } => {
                let l = {
                    let mut l = self.cur_len.write().unwrap();
                    *l += 1;
                    *l
                };
                self.cast.notify_each(*idx .. l);
            },
            VecDiff::Update{ idx, val: _ } => {
                self.cast.notify(&idx);
            }
        }
    }
}

impl<T> View for VecSequence<T>
where T: Clone + Send + Sync + 'static {
    type Msg = usize;
}

impl<T> SequenceView for VecSequence<T>
where T: Clone + Send + Sync + 'static {
    type Item = T;

    fn get(&self, idx: &usize) -> Option<T> {
        self.data.as_ref()?
            .read().unwrap()
            .get(*idx).cloned()
    }

    fn len(&self) -> Option<usize> {
        Some(*self.cur_len.read().unwrap())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct VecBuffer<T>
where T: Clone + Send + Sync + 'static {
    data: Arc<RwLock<Vec<T>>>,
    cast: Arc<RwLock<ObserverBroadcast<RwLock<Vec<T>>>>>
}

impl<T> VecBuffer<T>
where T: Clone + Send + Sync + 'static {
    pub fn with_data(
        data: Vec<T>,
        port: InnerViewPort<RwLock<Vec<T>>>
    ) -> Self {
        let data = Arc::new(RwLock::new(data));
        port.set_view(Some(data.clone()));
        VecBuffer { data, cast: port.get_broadcast() }
    }

    pub fn new(port: InnerViewPort<RwLock<Vec<T>>>) -> Self {
        VecBuffer::with_data(Vec::new(), port)
    }

    pub fn apply_diff(&mut self, diff: VecDiff<T>) {
        let mut data = self.data.write().unwrap();
        match &diff {
            VecDiff::Push(val) => { data.push(val.clone()); },
            VecDiff::Remove(idx) => { data.remove(*idx); },
            VecDiff::Insert{ idx, val } => { data.insert(*idx, val.clone()); },
            VecDiff::Update{ idx, val } => { data[*idx] = val.clone(); }
        }
        drop(data);
        self.cast.notify(&diff);
    }

    pub fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    pub fn get(&self, idx: usize) -> T {
        self.data.read().unwrap()[idx].clone()
    }

    pub fn push(&mut self, val: T) {
        self.apply_diff(VecDiff::Push(val));
    }

    pub fn remove(&mut self, idx: usize) {
        self.apply_diff(VecDiff::Remove(idx));
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Insert{ idx, val });
    }

    pub fn update(&mut self, idx: usize, val: T) {
        self.apply_diff(VecDiff::Update{ idx, val });
    }

    pub fn get_mut(&mut self, idx: usize) -> MutableVecAccess<T> {
        MutableVecAccess {
            buf: self.clone(),
            idx,
            val: self.get(idx)
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableVecAccess<T>
where T: Clone + Send + Sync + 'static {
    buf: VecBuffer<T>,
    idx: usize,
    val: T,
}

impl<T> Deref for MutableVecAccess<T>
where T: Clone + Send + Sync + 'static {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for MutableVecAccess<T>
where T: Clone + Send + Sync + 'static {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for MutableVecAccess<T>
where T: Clone + Send + Sync + 'static {
    fn drop(&mut self) {
        self.buf.update(self.idx, self.val.clone());
    }
}

