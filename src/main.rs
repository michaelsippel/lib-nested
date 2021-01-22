#![feature(trait_alias)]
#![feature(assoc_char_funcs)]

pub mod core;
pub mod index;
pub mod grid;
pub mod sequence;
pub mod singleton;
pub mod terminal;
pub mod projection;
pub mod string_editor;
pub mod leveled_term_view;

use {
    async_std::{task},
    std::{
        sync::{Arc, RwLock}
    },
    cgmath::{Vector2, Point2},
    termion::event::{Event, Key},
    crate::{
        core::{View, Observer, ObserverExt, ObserverBroadcast, ViewPort},
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
        string_editor::{StringEditor, insert_view::StringInsertView},
        leveled_term_view::LeveledTermView
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
        // string editor 1
        let mut editor = StringEditor::new();
        let (leveled_edit_view, leveled_edit_view_port) = LeveledTermView::new(editor.insert_view());
        compositor.push(
            leveled_edit_view_port
                .map_item(
                    move |_pos, atom| atom.add_style_back(
                        TerminalStyle::fg_color((200,200,200))))
        );

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // string editor 2
        let mut editor2 = StringEditor::new();
        let (leveled_edit2_view, leveled_edit2_view_port) = LeveledTermView::new(editor2.insert_view());
        compositor.push(
            leveled_edit2_view_port
                .map_item(
                    move |_pos, atom| atom.add_style_back(
                        TerminalStyle::fg_color((200,200,200))))
                .map_key(
                    |p| p + Vector2::new(0, 1),
                    |p| Some(p - Vector2::new(0, 1))
                )
        );

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // another view of the string, without editor
        compositor.push(
            editor.get_data_port()
                .to_sequence()
                .to_index()
                .map_key(
                    |idx| Point2::new(*idx as i16, 2 + *idx as i16),
                    |pt| if pt.x == pt.y-2 { Some(pt.x as usize) } else { None }
                ).map_item(
                    |_key, c| TerminalAtom::new(*c, TerminalStyle::fg_color((80, 20, 180)).add(TerminalStyle::bg_color((40,10,90))))
                )
        );

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // welcome message
        for c in "Welcome!".chars() {
            editor.insert(c);
            task::sleep(std::time::Duration::from_millis(80)).await;
        }

        task::sleep(std::time::Duration::from_millis(500)).await;

        for c in "Use arrow keys to navigate.".chars() {
            editor2.insert(c);
            task::sleep(std::time::Duration::from_millis(80)).await;
        }

        
                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/

        let mut sel = 0;

        leveled_edit_view.write().unwrap().set_level(if sel == 0 {1} else {0});
        leveled_edit2_view.write().unwrap().set_level(if sel == 1 {1} else {0});

        loop {
            let ed = match sel {
                0 => &mut editor,
                1 => &mut editor2,
                _ => &mut editor2
            };

            match term.next_event().await {
                TerminalEvent::Resize(size) => window_size.set(size),
                TerminalEvent::Input(Event::Key(Key::Up)) => {
                    sel = 0;

                    leveled_edit_view.write().unwrap().set_level(if sel == 0 {1} else {0});
                    leveled_edit2_view.write().unwrap().set_level(if sel == 1 {1} else {0});
                },
                TerminalEvent::Input(Event::Key(Key::Down)) => {
                    sel = 1;

                    leveled_edit_view.write().unwrap().set_level(if sel == 0 {1} else {0});
                    leveled_edit2_view.write().unwrap().set_level(if sel == 1 {1} else {0});
                },
                TerminalEvent::Input(Event::Key(Key::Left)) => ed.prev(),
                TerminalEvent::Input(Event::Key(Key::Right)) => ed.next(),
                TerminalEvent::Input(Event::Key(Key::Home)) => ed.goto(0),
                TerminalEvent::Input(Event::Key(Key::End)) => ed.goto_end(),
                TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {},
                TerminalEvent::Input(Event::Key(Key::Char(c))) => ed.insert(c),
                TerminalEvent::Input(Event::Key(Key::Delete)) => ed.delete(),
                TerminalEvent::Input(Event::Key(Key::Backspace)) => ed.delete_prev(),
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
            Some(TerminalAtom::new(self.0.chars().nth(pos.x as usize)?, TerminalStyle::fg_color((255, 255, 255))))
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

