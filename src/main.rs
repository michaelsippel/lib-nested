
#![feature(trait_alias)]

pub mod view;
pub mod port;
pub mod channel;

use {
    async_std::{
        prelude::*, task, stream
    },
    std::{
        sync::{Arc, RwLock}
    },
    cgmath::{Vector2},
    crate::{
        view::{View, Observer},
        port::{InnerViewPort, OuterViewPort}
    }
};

struct SingletonBuffer<T: Clone + Eq + Send + Sync + 'static> {
    data: Arc<RwLock<Option<T>>>,
    port: InnerViewPort<(), T>
}

impl<T: Clone + Eq + Send + Sync + 'static> SingletonBuffer<T> {
    fn new(
        port: InnerViewPort<(), T>
    ) -> Self {
        let data = Arc::new(RwLock::new(None));

        port.set_view_fn({
            let data = data.clone();
            move |new_val| data.read().unwrap().clone()
        });

        SingletonBuffer {
            data,
            port
        }
    }

    fn update(&mut self, new_value: T) {
        let mut data = self.data.write().unwrap();
        if *data != Some(new_value.clone()) {
            *data = Some(new_value);
            drop(data);
            self.port.notify(());
        }
    }
}

#[async_std::main]
async fn main() {
    let view_port = port::ViewPort::<(), char>::new();

    let mut buf = SingletonBuffer::new(view_port.inner());

    let view = view_port.outer().get_view();
    let mut stream = view_port.outer().stream().map({
        move |_| view.read().unwrap().as_ref().unwrap().view(()).unwrap()
    });

    let fut = task::spawn({
        async move {
            while let Some(val) = stream.next().await {
                println!("{}", val);
            }
            println!("end print task");
        }
    });

    buf.update('a');
    buf.update('b');
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.update('c');
    buf.update('d');
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.update('e');

    drop(buf);
    drop(view_port);

    fut.await;
}


