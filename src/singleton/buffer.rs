use {
    std::{
        sync::{Arc, RwLock}
    },
    crate::{
        core::{
            Observer,
            ObserverBroadcast,
            View,
            InnerViewPort
        },
        singleton::{SingletonView}
    }
};

pub struct SingletonBufferView<T: Clone + Eq + Send + Sync + 'static>(Arc<RwLock<T>>);

impl<T> View for SingletonBufferView<T>
where T: Clone + Eq + Send + Sync + 'static {
    type Msg = ();
}

impl<T> SingletonView for SingletonBufferView<T>
where T: Clone + Eq + Send + Sync + 'static {
    type Item = T;

    fn get(&self) -> Self::Item {
        self.0.read().unwrap().clone()
    }
}

#[derive(Clone)]
pub struct SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    value: Arc<RwLock<T>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SingletonView<Item = T>>>>
}

impl<T> SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    pub fn new(
        value: T,
        port: InnerViewPort<dyn SingletonView<Item = T>>
    ) -> Self {
        let value = Arc::new(RwLock::new(value));
        port.set_view(Some(Arc::new(SingletonBufferView(value.clone()))));

        SingletonBuffer {
            value,
            cast: port.get_broadcast()
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&mut self, new_value: T) {
        let mut v = self.value.write().unwrap();
        if *v != new_value {
            *v = new_value;
            drop(v);
            self.cast.notify(&());
        }
    }
}

// TODO: impl Deref & DerefMut

