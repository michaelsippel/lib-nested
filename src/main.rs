
#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod view;
pub mod port;
pub mod channel;
pub mod singleton_buffer;
pub mod vec_buffer;
pub mod terminal;
pub mod string_editor;

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
        terminal::{
            Terminal,
            TerminalAtom,
            TerminalStyle,
            TerminalCompositor,
            TerminalEvent
        }
    },
    termion::event::{Event, Key}
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
                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Setup Views
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/
        let fp = port::ViewPort::with_view(Arc::new(Fill(TerminalAtom::new('.', TerminalStyle::fg_color((50,50,50))))));
        compositor.push(fp.outer());

        let ep = port::ViewPort::new();
        let mut editor = string_editor::StringEditor::new(ep.inner());
        compositor.push(ep.outer());

                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/
        let mut term = Terminal::new();
        loop {
            match term.next_event().await {
                TerminalEvent::Resize(size) => {
                    for x in 0 .. size.x {
                        for y in 0 .. size.y {
                            fp.inner().notify(Vector2::new(x,y));
                        }
                    }
                },
                TerminalEvent::Input(Event::Key(Key::Left)) => editor.prev(),
                TerminalEvent::Input(Event::Key(Key::Right)) => editor.next(),
                TerminalEvent::Input(Event::Key(Key::Home)) => editor.goto(0),
                TerminalEvent::Input(Event::Key(Key::End)) => editor.goto_end(),
                TerminalEvent::Input(Event::Key(Key::Char(c))) => editor.insert(c),
                TerminalEvent::Input(Event::Key(Key::Delete)) => editor.delete(),
                TerminalEvent::Input(Event::Key(Key::Backspace)) => { editor.prev(); editor.delete(); },
                TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => break,
                _ => {}
            }
        }
    });

                        /*\
    <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Terminal Rendering
    <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        \*/
    Terminal::show(composite_view.into_outer()).await.ok();
}

