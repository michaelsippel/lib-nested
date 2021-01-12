#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod core;
pub mod index;
pub mod grid;
pub mod sequence;
pub mod singleton;
pub mod terminal;

pub mod string_editor;

use {
    async_std::{task},
    std::{
        sync::{Arc, RwLock},
        ops::Range
    },
    cgmath::{Vector2, Point2},
    termion::event::{Event, Key},
    crate::{
        core::{View, Observer, ObserverExt, ViewPort},
        index::{ImplIndexView},
        terminal::{
            TerminalView,
            TerminalAtom,
            TerminalStyle,
            TerminalEvent,
            Terminal,
            TerminalCompositor
        },
        grid::GridOffset
    }
};

struct VecSequenceView<T: Send + Sync + Clone>(Arc<RwLock<Vec<T>>>);
impl<T: Send + Sync + Clone> ImplIndexView for VecSequenceView<T> {
    type Key = usize;
    type Value = T;

    fn get(&self, idx: &usize) -> T {
        self.0.read().unwrap()[*idx].clone()
    }

    fn range(&self) -> Option<Range<usize>> {
        Some(0 .. self.0.read().unwrap().len())
    }
}

struct Checkerboard;
impl ImplIndexView for Checkerboard {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        if pos.x == 0 || pos.x == 1 || pos.x > 17 || pos.y == 0 || pos.y > 8 {
            // border
            Some(TerminalAtom::new_bg((20, 10, 10)))
        } else {
            // field
            if ((pos.x/2) % 2 == 0) ^ ( pos.y % 2 == 0 ) {
                Some(TerminalAtom::new_bg((0, 0, 0)))
            } else {
                Some(TerminalAtom::new_bg((200, 200, 200)))
            }
        }
    }

    fn range(&self) -> Option<Range<Point2<i16>>> {
        Some(Point2::new(0,0) .. Point2::new(20,10))
    }
}

struct ScrambleBackground;
impl ImplIndexView for ScrambleBackground {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        if ((pos.x/2) % 2 == 0) ^ (pos.y % 2 == 0) {
            Some(TerminalAtom::new(char::from((35+(5*pos.y+pos.x)%40) as u8), TerminalStyle::fg_color((40, 40, 40))))
        } else {
            Some(TerminalAtom::new(char::from((35+(pos.y+9*pos.x)%40) as u8), TerminalStyle::fg_color((90, 90, 90))))
        }
    }

    fn range(&self) -> Option<Range<Point2<i16>>> {
        None
        //Some(Point2::new(0,0) .. Point2::new(50,30))
    }
}

#[async_std::main]
async fn main() {
    let term_port = ViewPort::<dyn TerminalView>::new();

    let mut compositor = TerminalCompositor::new(term_port.inner());
    compositor.push(ViewPort::<dyn TerminalView>::with_view(Arc::new(ScrambleBackground)).into_outer());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    task::spawn(async move {
                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Setup Views
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/

        let offset_port = ViewPort::<dyn TerminalView>::new();
        let o = GridOffset::new(offset_port.inner());

        let checkerboard_port = ViewPort::<dyn TerminalView>::with_view(Arc::new(Checkerboard));
        checkerboard_port.add_observer(o.clone());

        compositor.push(offset_port.into_outer());

        let edit_port = ViewPort::<dyn TerminalView>::new();        
        let mut editor = string_editor::StringEditor::new(edit_port.inner());

        let edit_offset_port = ViewPort::<dyn TerminalView>::new();
        let edit_o = GridOffset::new(edit_offset_port.inner());
        edit_port.add_observer(edit_o.clone());

        compositor.push(
            edit_offset_port
                .into_outer()
                // add a nice black background
                .map_item(|atom| atom.map(
                         |a| a.add_style_back(TerminalStyle::bg_color((0,0,0))))));

        edit_o.write().unwrap().set_offset(Vector2::new(40, 4));

        task::spawn(async move {
            for x in 0 .. 20 {
                async_std::task::sleep(std::time::Duration::from_millis(15)).await;
                o.write().unwrap().set_offset(Vector2::new(x as i16, x as i16));
            }
        });
        
                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/
        loop {
            match term.next_event().await {
                TerminalEvent::Input(Event::Key(Key::Left)) => editor.prev(),
                TerminalEvent::Input(Event::Key(Key::Right)) => editor.next(),
                TerminalEvent::Input(Event::Key(Key::Home)) => editor.goto(0),
                TerminalEvent::Input(Event::Key(Key::End)) => editor.goto_end(),
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {},
                TerminalEvent::Input(Event::Key(Key::Char(c))) => editor.insert(c),
                TerminalEvent::Input(Event::Key(Key::Delete)) => editor.delete(),
                TerminalEvent::Input(Event::Key(Key::Backspace)) => { editor.prev(); editor.delete(); },
                TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => {
                    break
                }
                _ => {}
            }
        }
    });

                        /*\
    <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Terminal Rendering
    <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        \*/
    term_writer.show().await.ok();
}

