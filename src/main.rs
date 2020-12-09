
#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod view;
pub mod port;
pub mod channel;
pub mod singleton_buffer;
pub mod vec_buffer;
pub mod terminal;

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
        vec_buffer::VecBuffer,
        terminal::{Terminal, TerminalAtom, TerminalStyle}
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
                Some(TerminalAtom::new(char::from_digit(digit, 16).unwrap(), TerminalStyle::bg_color((100,30,30))))
            } else {
                None
            }
        )
        // simple horizontal layout
        .map_key(
            |idx| Vector2::<i16>::new(idx as i16, 0),
            |pos| pos.x as usize
        );

    let fut = task::spawn(Terminal::show(digit_view));

    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.push(0);
    buf.push(10);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.push(2);
    buf.push(3);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.push(4);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.insert(0, 15);
    task::sleep(std::time::Duration::from_secs(1)).await;
    buf.remove(2);
    task::sleep(std::time::Duration::from_secs(1)).await;

    drop(buf);
    drop(digits);

    fut.await;
}

