
#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

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
            move |_| data.read().unwrap().clone()
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



impl<T: Clone + Send + Sync> View for Vec<T> {
    type Key = usize;
    type Value = T;

    fn view(&self, key: usize) -> Option<T> {
        self.get(key).cloned()
    }
}


struct VecBuffer<T: Clone + Eq + Send + Sync + 'static> {
    data: Arc<RwLock<Vec<T>>>,
    port: InnerViewPort<usize, T>
}

impl<T: Clone + Eq + Send + Sync + 'static> VecBuffer<T> {
    fn new(port: InnerViewPort<usize, T>) -> Self {
        let data = Arc::new(RwLock::new(Vec::new()));
        port.set_view(data.clone());
        VecBuffer { data, port }
    }

    fn push(&mut self, val: T) {
        self.port.notify({
            let mut d = self.data.write().unwrap();
            let len = d.len();
            d.push(val);
            len
        });
    }
}

#[async_std::main]
async fn main() {
    let digits = port::ViewPort::new();
    let mut buf = VecBuffer::new(digits.inner());

    let digit_view = digits.outer()

        // digit encoding
        .map_value(
            |digit|
            if let Some(digit) = digit {
                char::from_digit(digit, 16)
            } else {
                None
            }
        )

        // simple horizontal layout
        .map_key(
            |idx| Vector2::<i16>::new(idx as i16, 0),
            |pos| pos.x as usize
        );

    let view = digit_view.get_view();
    let mut stream = digit_view.stream().map({
        move |idx| (idx, view.view(idx))
    });

    let fut = task::spawn({
        async move {
            while let Some((idx, val)) = stream.next().await {
                println!("v[{:?}] = {:?}", idx, val);
            }
            println!("end print task");
        }
    });

    buf.push(0);
    buf.push(1);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.push(2);
    buf.push(3);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.push(4);

    drop(buf);
    drop(digits);
    drop(digit_view);

    fut.await;
}


