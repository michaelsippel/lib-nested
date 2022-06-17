use {
    crate::{
        core::{InnerViewPort, OuterViewPort, Observer, ObserverBroadcast, View, ViewPort},
        singleton::SingletonView,
    },
    std::sync::RwLock,
    std::{
        ops::{Deref, DerefMut},
        sync::Arc,
    },
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct SingletonBufferView<T: Clone + Send + Sync + 'static>(Arc<RwLock<T>>);

impl<T> View for SingletonBufferView<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Msg = ();
}

impl<T> SingletonView for SingletonBufferView<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    fn get(&self) -> Self::Item {
        self.0.read().unwrap().clone()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct SingletonBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    value: Arc<RwLock<T>>,
    port: InnerViewPort<dyn SingletonView<Item = T>>
}

impl<T> SingletonBuffer<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn with_port(value: T, port: InnerViewPort<dyn SingletonView<Item = T>>) -> Self {
        let value = Arc::new(RwLock::new(value));
        port.set_view(Some(Arc::new(SingletonBufferView(value.clone()))));

        SingletonBuffer {
            value,
            port
        }
    }

    pub fn new(value: T) -> Self {
        SingletonBuffer::with_port(value, ViewPort::new().into_inner())
    }

    pub fn get_port(&self) -> OuterViewPort<dyn SingletonView<Item = T>> {
        self.port.0.outer()
    }
    
    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn get_mut(&self) -> MutableSingletonAccess<T> {
        MutableSingletonAccess {
            buf: self.clone(),
            val: self.get(),
        }
    }

    pub fn set(&mut self, new_value: T) {
        let mut v = self.value.write().unwrap();
        *v = new_value;
        drop(v);
        self.port.notify(&());
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    buf: SingletonBuffer<T>,
    val: T,
}

impl<T> Deref for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> DerefMut for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T> Drop for MutableSingletonAccess<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.buf.set(self.val.clone());
    }
}
