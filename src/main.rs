
#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod view;
pub mod port;
pub mod channel;
pub mod singleton_buffer;
pub mod vec_buffer;
pub mod terminal;

use {
    async_std::{task},
    std::{
        sync::{Arc, RwLock}
    },
    cgmath::{Vector2},
    crate::{
        view::{View, Observer},
        port::{ViewPort, InnerViewPort, OuterViewPort},
        singleton_buffer::SingletonBuffer,
        vec_buffer::VecBuffer,
        terminal::{Terminal, TerminalAtom, TerminalStyle, TerminalCompositor}
    }
};

struct Fill(TerminalAtom);
impl View for Fill {
    type Key = Vector2<i16>;
    type Value = TerminalAtom;

    fn view(&self, _: Vector2<i16>) -> Option<TerminalAtom> {
        Some(self.0.clone())
    }
}

#[async_std::main]
async fn main() {
    let composite_view = port::ViewPort::new();
    let mut compositor = TerminalCompositor::new(composite_view.inner());

    task::spawn(async move {
        // background
        let fp = port::ViewPort::with_view(Arc::new(Fill(TerminalAtom::new('.', TerminalStyle::fg_color((50,50,50))))));
        compositor.push(fp.outer());

        // view of Vec<u32>
        let digits = port::ViewPort::new();
        let mut buf = VecBuffer::new(digits.inner());
        compositor.push(
            digits.outer()
                .map_value( // digit encoding
                    |digit|
                    if let Some(digit) = digit {
                        Some(TerminalAtom::new(
                            char::from_digit(digit, 16).unwrap(),
                            TerminalStyle::bg_color((100,30,30)).add(
                                TerminalStyle::fg_color((255,255,255)))))
                    } else {
                        None
                    }
                )
                .map_key( // a lightly tilted layout
                    // mapping from index to position in 2D-grid
                    |idx| Vector2::<i16>::new(idx as i16, idx as i16 / 2),
                    // reverse mapping from position to idx
                    |pos| pos.x as usize
                ));

        // TODO: use the real terminal size...
        for x in 0 .. 10 {
            for y in 0 .. 10 {
                fp.inner().notify(Vector2::new(x,y));
            }
        }

        // now some modifications on our VecBuffer, which will automatically update the View
        buf.push(0);
        buf.push(10);
        task::sleep(std::time::Duration::from_millis(400)).await;
        buf.push(2);
        buf.push(3);
        task::sleep(std::time::Duration::from_millis(400)).await;
        buf.push(4);
        task::sleep(std::time::Duration::from_millis(400)).await;
        buf.insert(0, 15);
        task::sleep(std::time::Duration::from_millis(400)).await;
        buf.remove(2);
        task::sleep(std::time::Duration::from_millis(400)).await;

        for _ in 0 .. 4 {
            buf.remove(0);
            task::sleep(std::time::Duration::from_millis(400)).await;
        }
    });

    Terminal::show(composite_view.into_outer()).await.ok();
}

