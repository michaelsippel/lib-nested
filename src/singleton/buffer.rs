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

pub struct SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    value: T,
    cast: Arc<RwLock<ObserverBroadcast<dyn SingletonView<Item = T>>>>
}

impl<T> View for SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    type Msg = ();
}

impl<T> SingletonView for SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    type Item = T;

    fn get(&self) -> Self::Item {
        self.value.clone()
    }
}

impl<T> SingletonBuffer<T>
where T: Clone + Eq + Send + Sync + 'static {
    pub fn new(
        value: T,
        port: InnerViewPort<dyn SingletonView<Item = T>>
    ) -> Arc<RwLock<Self>> {
        let buf = Arc::new(RwLock::new(
            SingletonBuffer {
                value,
                cast: port.get_broadcast()
            }
        ));
        port.set_view(Some(buf.clone()));
        buf
    }

    pub fn set(&mut self, new_value: T) {
        if self.value != new_value {
            self.value = new_value;
            self.cast.notify(&());
        }
    }
}

// TODO: impl Deref & DerefMut

