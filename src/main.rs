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
        sync::{Arc, RwLock}
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
        grid::{GridOffset, GridWindowIterator},
        singleton::{SingletonView, SingletonBuffer},
        string_editor::{StringEditor}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

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

        let window_size_port = ViewPort::new();
        let mut window_size = SingletonBuffer::new(Vector2::new(0, 0), window_size_port.inner());

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // string editor
        let mut editor = StringEditor::new();

        let edit_view_port = ViewPort::new();
        let edit_view =
            string_editor::insert_view::StringEditView::new(
                editor.get_cursor_port(),
                editor.get_data_port().to_sequence(),
                edit_view_port.inner()
            );

        compositor.push(
            edit_view_port.outer()
                .map_item(
                    |_pos, atom| atom.add_style_back(
                        TerminalStyle::fg_color((200,200,200))
                            .add(TerminalStyle::bg_color((0,0,0)))
                            .add(TerminalStyle::bold(true)))
                )
        );

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // another view of the string, without editor
        compositor.push(
            editor.get_data_port()
                .to_sequence()
                .to_index()
                .map_key(
                    |idx| Point2::new(*idx as i16, 2),
                    |pt| if pt.y == 2 { Some(pt.x as usize) } else { None }
                ).map_item(
                    |_key, c| TerminalAtom::new(*c, TerminalStyle::fg_color((0, 200, 0)))
                )
        );

                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/
        loop {
            match term.next_event().await {
                TerminalEvent::Resize(size) => window_size.set(size),
                TerminalEvent::Input(Event::Key(Key::Left)) => editor.prev(),
                TerminalEvent::Input(Event::Key(Key::Right)) => editor.next(),
                TerminalEvent::Input(Event::Key(Key::Home)) => editor.goto(0),
                TerminalEvent::Input(Event::Key(Key::End)) => editor.goto_end(),
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {},
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

    term_writer.show().await.ok();
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
struct Checkerboard;
impl ImplIndexView for Checkerboard {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

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

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(Point2::new(0,0) .. Point2::new(20,10)).collect())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
struct TermLabel(String);
impl ImplIndexView for TermLabel {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        if pos.y == 5 {
            Some(TerminalAtom::from(self.0.chars().nth(pos.x as usize)?))
        } else {
            None
        }
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(
            GridWindowIterator::from(
                Point2::new(0, 5) .. Point2::new(self.0.chars().count() as i16, 6)
            ).collect()
        )
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
struct ScrambleBackground;
impl ImplIndexView for ScrambleBackground {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        if ((pos.x/2) % 2 == 0) ^ (pos.y % 2 == 0) {
            Some(TerminalAtom::new(char::from((35+(5*pos.y+pos.x)%40) as u8), TerminalStyle::fg_color((40, 40, 40))))
        } else {
            Some(TerminalAtom::new(char::from((35+(pos.y+9*pos.x)%40) as u8), TerminalStyle::fg_color((90, 90, 90))))
        }
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        None
    }
}


