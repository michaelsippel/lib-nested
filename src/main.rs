
#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod view;
pub mod port;
pub mod channel;
pub mod singleton_buffer;
pub mod vec_buffer;

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
        port::{InnerViewPort, OuterViewPort},
        singleton_buffer::SingletonBuffer,
        vec_buffer::VecBuffer
    }
};

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


