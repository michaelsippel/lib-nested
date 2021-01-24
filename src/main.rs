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
pub mod cell_layout;

use {
    async_std::{task},
    std::{
        sync::{Arc, RwLock}
    },
    cgmath::{Vector2, Point2},
    termion::event::{Event, Key},
    crate::{
        core::{View, Observer, ObserverExt, ObserverBroadcast, ViewPort, OuterViewPort},
        index::{ImplIndexView},
        terminal::{
            TerminalView,
            TerminalAtom,
            TerminalStyle,
            TerminalEvent,
            Terminal,
            TerminalCompositor
        },
        sequence::{VecBuffer, SequenceView},
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

        let opening_port = ViewPort::new();
        let mut opening = VecBuffer::new(opening_port.inner());

        let delim_port = ViewPort::new();
        let mut delim = VecBuffer::new(delim_port.inner());

        let closing_port = ViewPort::new();
        let mut closing = VecBuffer::new(closing_port.inner());

        let e1_port = ViewPort::new();
        let mut e1 = VecBuffer::new(e1_port.inner());

        let e2_port = ViewPort::new();
        let mut e2 = VecBuffer::new(e2_port.inner());

        opening.push(TerminalAtom::new('[', TerminalStyle::fg_color((180, 120, 80))));
        delim.push(TerminalAtom::new(',', TerminalStyle::fg_color((180, 120, 80))));
        delim.push(TerminalAtom::new(' ', TerminalStyle::fg_color((180, 120, 80))));
        closing.push(TerminalAtom::new(']', TerminalStyle::fg_color((180, 120, 80))));

        let str_list_port = ViewPort::new();
        let mut str_list = VecBuffer::<OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>::new(str_list_port.inner());

        str_list.push(opening_port.outer().to_sequence());
        str_list.push(closing_port.outer().to_sequence());

        compositor.push(
            str_list_port.outer()
                .to_sequence()
                .flatten()
                .to_index()
                .map_key(
                    |idx| Point2::new(*idx as i16, 0 as i16),
                    |pt| if pt.y == 0 { Some(pt.x as usize) } else { None }
                )
        );

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // welcome message
        task::sleep(std::time::Duration::from_millis(500)).await;
        str_list.insert(1, e1_port.outer().to_sequence());
        for c in "Welcome!".chars() {
            e1.push(TerminalAtom::new(c, TerminalStyle::fg_color((180, 180, 255))));
            task::sleep(std::time::Duration::from_millis(80)).await;
        }
        task::sleep(std::time::Duration::from_millis(500)).await;
        str_list.insert(2, delim_port.outer().to_sequence());
        str_list.insert(3, e2_port.outer().to_sequence());
        task::sleep(std::time::Duration::from_millis(80)).await;
        for c in "This is a flattened SequenceView.".chars() {
            e2.push(TerminalAtom::new(c, TerminalStyle::fg_color((180, 180, 255))));
            task::sleep(std::time::Duration::from_millis(80)).await;
        }

        task::sleep(std::time::Duration::from_millis(500)).await;

        let l2_port = ViewPort::new();
        let mut l2 = VecBuffer::new(l2_port.inner());

        *str_list.get_mut(1) = l2_port.outer().to_sequence().flatten();

        l2.push(opening_port.outer().to_sequence());

        e1.clear();
        l2.push(e1_port.outer().to_sequence());
        l2.push(closing_port.outer().to_sequence());

        for c in "they can even be NeStEd!".chars() {
            e1.push(TerminalAtom::new(c, TerminalStyle::fg_color((180, 180, 255))));
            task::sleep(std::time::Duration::from_millis(80)).await;
        }

        for i in 0 .. 10 {
            task::sleep(std::time::Duration::from_millis(100)).await;

            let col = (100+10*i, 55+20*i, 20+ 20*i);
            *opening.get_mut(0) = TerminalAtom::new('{', TerminalStyle::fg_color(col));
            *closing.get_mut(0) = TerminalAtom::new('}', TerminalStyle::fg_color(col));
        }

        for i in 0 .. 10 {
            task::sleep(std::time::Duration::from_millis(100)).await;

            let col = (100+10*i, 55+20*i, 20+ 20*i);
            *opening.get_mut(0) = TerminalAtom::new('<', TerminalStyle::fg_color(col));
            *closing.get_mut(0) = TerminalAtom::new('>', TerminalStyle::fg_color(col));
        }

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // string editor 1
        let mut editor = StringEditor::new();
        let (leveled_edit_view, leveled_edit_view_port) = LeveledTermView::new(editor.insert_view());

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // string editor 2
        let mut editor2 = StringEditor::new();
        let (leveled_edit2_view, leveled_edit2_view_port) = LeveledTermView::new(editor2.insert_view());

        
                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/

        let mut sel = 0 as usize;

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

