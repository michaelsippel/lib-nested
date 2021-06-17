use {
    std::{
        sync::Arc,
    },
    std::sync::RwLock,
    crate::{
        core::{View, Observer, ObserverExt, ObserverBroadcast, ViewPort, InnerViewPort, OuterViewPort},
        sequence::SequenceView,
        vec::VecDiff
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Adapter View implementing `Sequence` for `Vec`
pub struct VecSequence<T>
where T: Clone + Send + Sync + 'static {
    cur_len: usize,
    data: Option<Arc<RwLock<Vec<T>>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = T>>>>
}


impl<T> VecSequence<T>
where T: Clone + Send + Sync + 'static {
    pub fn new(
        port: InnerViewPort<dyn SequenceView<Item = T>>
    ) -> Arc<RwLock<Self>> {
        let seq = Arc::new(RwLock::new(
            VecSequence {
                cur_len: 0,
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
        let old_len = self.cur_len;
        self.data = view;
        self.cur_len = 0;
        /*
            if let Some(data) = self.data.as_ref() {
                data.read().unwrap().len()
            } else {
                0
            };
*/
        self.cast.notify_each(0 .. std::cmp::max(old_len, self.cur_len));
    }

    fn notify(&mut self, diff: &VecDiff<T>) {
        match diff {
            VecDiff::Clear => {
                self.cast.notify_each(0 .. self.cur_len);
                self.cur_len = 0
            },
            VecDiff::Push(_) => {
                self.cast.notify(&self.cur_len);
                self.cur_len += 1;
            },
            VecDiff::Remove(idx) => {
                self.cast.notify_each(*idx .. self.cur_len);
                self.cur_len -= 1;
            },
            VecDiff::Insert{ idx, val: _ } => {
                self.cur_len += 1;
                self.cast.notify_each(*idx .. self.cur_len);
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
        Some(self.cur_len)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<T> OuterViewPort<RwLock<Vec<T>>>
where T: Clone + Send + Sync + 'static {
    pub fn to_sequence(&self) -> OuterViewPort<dyn SequenceView<Item = T>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let vec_seq = VecSequence::new(port.inner());
        self.add_observer(vec_seq.clone());
        port.into_outer()
    }
}


