
#![feature(trait_alias)]

pub mod view;
pub mod port;
pub mod channel;

use async_std::task;
use async_std::prelude::*;
use async_std::stream;

use std::sync::{Arc, RwLock};
use cgmath::Vector2;

use view::{View, Observer};
use port::{ViewPortIn, ViewPortOut};

#[async_std::main]
async fn main() {
    let (view_in, mut view_out) = port::view_port::<usize, char>();

    let mut observer_stream = view_in.stream().map({
        let view = view_in.clone();
        move |idx| (idx, view.view(idx).unwrap())
    });

    let fut = task::spawn(async move {
        while let Some((idx, val)) = observer_stream.next().await {
            println!("view[{}] = {}", idx, val);
        }
    });

    view_out.set_view_fn(|idx| Some(if idx % 2 == 0 { 'λ' } else { 'y' }) );

    view_out.notify(1);
    view_out.notify(2);
    view_out.notify(5);

    fut.await;
}


