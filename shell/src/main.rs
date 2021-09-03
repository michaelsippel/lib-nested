
extern crate tty;

mod monstera;
mod process;

use{
    std::sync::{Arc, RwLock},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            OuterViewPort,
            Observer,
            ObserverExt,
            context::{ReprTree, Object, MorphismType, MorphismMode, Context},
            port::{UpdateTask}},
        index::{IndexView},
        sequence::{SequenceView, SequenceViewExt},
        vec::{VecBuffer},
        integer::{RadixProjection, DigitEditor, PosIntEditor},
        terminal::{
            Terminal,
            TerminalStyle,
            TerminalAtom,
            TerminalCompositor,
            TerminalEvent,
            make_label,
            TerminalView,
            TerminalEditor},
        string_editor::{StringEditor},
        tree_nav::{TreeNav, TreeNavResult, TreeCursor, TerminalTreeEditor},
        list::{SExprView, ListCursorMode, ListEditor, ListEditorStyle}
    },
    crate::{
        process::ProcessLauncher
    }
};

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    async_std::task::spawn(
        async move {
            let table_port = ViewPort::<dyn nested::grid::GridView<Item = OuterViewPort<dyn TerminalView>>>::new();
            let mut table_buf = nested::index::buffer::IndexBuffer::new(table_port.inner());

            let magic = make_label("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>")
                .map_item(
                    |pos, atom|
                    atom.add_style_back(
                        TerminalStyle::fg_color(
                            (5,
                             ((80+(pos.x*30)%100) as u8),
                             (55+(pos.x*15)%180) as u8)
                        )
                    )
                );

            let cur_size_port = ViewPort::new();
            let mut cur_size = nested::singleton::SingletonBuffer::new(Vector2::new(10, 10), cur_size_port.inner());

            let status_chars_port = ViewPort::new();
            let mut status_chars = VecBuffer::new(status_chars_port.inner());

            table_buf.insert_iter(vec![
                (Point2::new(0, 0), magic.clone()),
                (Point2::new(0, 1), status_chars_port.outer().to_sequence().to_grid_horizontal()),
            ]);

            compositor.write().unwrap().push(monstera::make_monstera());
            compositor.write().unwrap().push(table_port.outer().flatten().offset(Vector2::new(40, 2)));

            let mut y = 2;

            let mut process_launcher = ProcessLauncher::new();
            table_buf.insert(Point2::new(0, y), process_launcher.get_term_view());

            process_launcher.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![ 0 ]
            });

            loop {
                term_port.update();
                match term.next_event().await {
                    TerminalEvent::Resize(new_size) => {
                        cur_size.set(new_size);
                        term_port.inner().get_broadcast().notify_each(
                            nested::grid::GridWindowIterator::from(
                                Point2::new(0,0) .. Point2::new(new_size.x, new_size.y)
                            )
                        );
                    }
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                    TerminalEvent::Input(Event::Key(Key::Left)) => {
                        process_launcher.pxev();
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        process_launcher.nexd();
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => { process_launcher.up(); }
                    TerminalEvent::Input(Event::Key(Key::Down)) => {
                        if process_launcher.dn() == TreeNavResult::Continue {
                            process_launcher.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        process_launcher.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        process_launcher.goto_end();
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        let output_view = process_launcher.launch();

                        y += 1;
                        table_buf.insert(Point2::new(0, y), output_view);

                        process_launcher = ProcessLauncher::new();
                        process_launcher.goto(TreeCursor {
                            leaf_mode: ListCursorMode::Insert,
                            tree_addr: vec![ 0 ]
                        });

                        y += 1;
                        table_buf.insert(Point2::new(0, y), process_launcher.get_term_view());
                    }

                    ev => {
                        if process_launcher.get_cursor().leaf_mode == ListCursorMode::Select {
                            match ev {
                                TerminalEvent::Input(Event::Key(Key::Char('l'))) => { process_launcher.up(); },
                                TerminalEvent::Input(Event::Key(Key::Char('a'))) => { process_launcher.dn(); },
                                TerminalEvent::Input(Event::Key(Key::Char('i'))) => { process_launcher.pxev(); },
                                TerminalEvent::Input(Event::Key(Key::Char('e'))) => { process_launcher.nexd(); },
                                TerminalEvent::Input(Event::Key(Key::Char('u'))) => { process_launcher.goto_home(); },
                                TerminalEvent::Input(Event::Key(Key::Char('o'))) => { process_launcher.goto_end(); },
                                _ => {
                                    process_launcher.handle_terminal_event(&ev);
                                }
                            }
                        } else {
                            process_launcher.handle_terminal_event(&ev);
                        }
                    }
                }

                status_chars.clear();
                let cur = process_launcher.get_cursor();

                if cur.tree_addr.len() > 0 {
                    status_chars.push(TerminalAtom::new('@', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                    for x in cur.tree_addr {
                        for c in format!("{}", x).chars() {
                            status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((0, 100, 20))));
                        }
                        status_chars.push(TerminalAtom::new('.', TerminalStyle::fg_color((120, 80, 80))));
                    }

                    status_chars.push(TerminalAtom::new(':', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                    for c in
                        match cur.leaf_mode {
                            ListCursorMode::Insert => "INSERT",
                            ListCursorMode::Select => "SELECT",
                            ListCursorMode::Modify => "MODIFY"
                        }.chars()
                    {
                        status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((200, 200, 20))));
                    }
                    status_chars.push(TerminalAtom::new(':', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                } else {
                    for c in "Press <DN> to enter".chars() {
                        status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((200, 200, 20))));
                    }
                }
            }

            //drop(term);
        }
    );

    term_writer.show().await.expect("output error!");
}

