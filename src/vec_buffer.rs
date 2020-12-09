use {
    std::{
        sync::{Arc, RwLock}
    },
    crate::{
        view::{View, Observer},
        port::{InnerViewPort}
    }
};

impl<T: Clone + Send + Sync> View for Vec<T> {
    type Key = usize;
    type Value = T;

    fn view(&self, key: usize) -> Option<T> {
        self.get(key).cloned()
    }
}

pub struct VecBuffer<T: Clone + Eq + Send + Sync + 'static> {
    data: Arc<RwLock<Vec<T>>>,
    port: InnerViewPort<usize, T>
}

impl<T: Clone + Eq + Send + Sync + 'static> VecBuffer<T> {
    pub fn new(port: InnerViewPort<usize, T>) -> Self {
        let data = Arc::new(RwLock::new(Vec::new()));
        port.set_view(data.clone());
        VecBuffer { data, port }
    }

    pub fn push(&mut self, val: T) {
        self.port.notify({
            let mut d = self.data.write().unwrap();
            let idx = d.len();
            d.push(val);
            idx
        });
    }

    pub fn remove(&mut self, idx: usize) {
        let len = {
            let mut d = self.data.write().unwrap();
            let len = d.len();
            d.remove(idx);
            len
        };

        for i in idx .. len {
            self.port.notify(i);
        }
    }

    pub fn insert(&mut self, idx: usize, val: T) {
        let len = {
            let mut d = self.data.write().unwrap();
            d.insert(idx, val);
            let len = d.len();
            len
        };

        for i in idx .. len {
            self.port.notify(i);
        }
    }
}

