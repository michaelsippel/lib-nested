use {
    std::{
        sync::{Arc, RwLock}
    },
    crate::{
        view::Observer,
        port::InnerViewPort
    }
};

pub struct SingletonBuffer<T: Clone + Eq + Send + Sync + 'static> {
    data: Arc<RwLock<Option<T>>>,
    port: InnerViewPort<(), T>
}

impl<T: Clone + Eq + Send + Sync + 'static> SingletonBuffer<T> {
    pub fn new(
        port: InnerViewPort<(), T>
    ) -> Self {
        let data = Arc::new(RwLock::new(None));

        port.set_view_fn({
            let data = data.clone();
            move |_| data.read().unwrap().clone()
        });

        SingletonBuffer {
            data,
            port
        }
    }

    pub fn update(&mut self, new_value: T) {
        let mut data = self.data.write().unwrap();
        if *data != Some(new_value.clone()) {
            *data = Some(new_value);
            drop(data);
            self.port.notify(());
        }
    }
}

