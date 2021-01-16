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
        singleton::{SingletonView, SingletonBuffer}
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
        let window_size = SingletonBuffer::new(Vector2::new(0, 0), window_size_port.inner());

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // string editor
        let edit_port = ViewPort::<dyn TerminalView>::new();        
        let mut editor = string_editor::StringEditor::new(edit_port.inner());

        compositor.push(edit_port.outer().map_key(
            |pt| pt + Vector2::new(4, 2),
            |pt| Some(pt - Vector2::new(4, 2))
        ));

        let edit_offset_port = ViewPort::<dyn TerminalView>::new();
        let edit_o = GridOffset::new(edit_offset_port.inner());

        edit_port.add_observer(edit_o.clone());

        compositor.push(
            edit_offset_port
                .into_outer()
                // add a nice black background
                .map_item(|atom| atom.map(
                    |a| a.add_style_back(TerminalStyle::bg_color((0,0,0)))))
        );

        edit_o.write().unwrap().set_offset(Vector2::new(40, 4));

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // stupid label animation
        let label_port = ViewPort::<dyn TerminalView>::new();
        compositor.push(
            label_port.outer()
                .map_item(
                    |atom| atom.map(|atom|
                                    atom.add_style_back(TerminalStyle::fg_color((255, 255, 255)))
                                    .add_style_back(TerminalStyle::bg_color((0, 0, 0))))
                )
        );
        task::spawn(async move {
            loop {
                label_port.set_view(Some(Arc::new(TermLabel(String::from("Hello")))));
                task::sleep(std::time::Duration::from_secs(1)).await;
                label_port.set_view(Some(Arc::new(TermLabel(String::from("I'm a dynamic label")))));
                task::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        //<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
        // Vec-Buffer
        let vec_port = ViewPort::new();
        let mut vec_buf = sequence::VecBuffer::<char>::new(vec_port.inner());

        // project Vec-Buffer to SequenceView
        let vec_seq_port = ViewPort::new();
        let vec_seq = sequence::VecSequence::new(vec_seq_port.inner());
        vec_port.add_observer(vec_seq.clone());
        let vec_term_view = vec_seq_port.outer()
            .to_index()
            .map_key(
                |idx: &usize| Point2::<i16>::new(*idx as i16, 0),
                |pt: &Point2<i16>| if pt.y == 0 { Some(pt.x as usize) } else { None }
            )
            .map_item(
                |c| Some(TerminalAtom::new(c.clone()?, TerminalStyle::fg_color((200, 10, 10))))
            );

        compositor.push(vec_term_view);

        vec_buf.push('a');
        vec_buf.push('b');
        vec_buf.push('c');

                            /*\
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                        Event Loop
        <<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                            \*/
        loop {
            match term.next_event().await {
                TerminalEvent::Resize(size) => window_size.write().unwrap().set(size),
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

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(Point2::new(0,0) .. Point2::new(20,10)).collect())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
struct TermLabel(String);
impl ImplIndexView for TermLabel {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

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
    type Value = Option<TerminalAtom>;

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


