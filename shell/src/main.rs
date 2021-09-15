
extern crate portable_pty;

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
        grid::{GridWindowIterator},
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


struct AsciiBox {
    content: Option<Arc<dyn TerminalView>>,
    extent: Vector2<i16>
}

impl View for AsciiBox {
    type Msg = Point2<i16>;
}

impl IndexView<Point2<i16>> for AsciiBox {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        if pt.x == 0 || pt.x == self.extent.x {
            // vertical line
            if pt.y == 0 && pt.x == 0 {
                Some(TerminalAtom::from('╭'))
            } else if pt.y == 0 && pt.x == self.extent.x {
                Some(TerminalAtom::from('╮'))
            } else if pt.y > 0 && pt.y < self.extent.y {
                Some(TerminalAtom::from('│'))
            } else if pt.y == self.extent.y && pt.x == 0 {
                Some(TerminalAtom::from('╰'))
            } else if pt.y == self.extent.y && pt.x == self.extent.x {
                Some(TerminalAtom::from('╯'))
            } else {                
                None
            }
        } else if pt.y == 0 || pt.y == self.extent.y {
            // horizontal line
            if pt.x > 0 && pt.x < self.extent.x {
                Some(TerminalAtom::from('─'))
            } else {
                None
            }
        } else if
            pt.x < self.extent.x &&
            pt.y < self.extent.y
        {
            self.content.get(&(pt - Vector2::new(1, 1)))
        } else {
            None
        }
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(
            Point2::new(0, 0) ..= Point2::new(self.extent.x, self.extent.y)
        ).collect())
    }
}

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
                (Point2::new(0, 2), magic.clone()),
            ]);

            //compositor.write().unwrap().push(monstera::make_monstera());
            compositor.write().unwrap().push(table_port.outer().flatten());//.offset(Vector2::new(40, 2)));

            let mut y = 4;

            let mut process_launcher = ProcessLauncher::new();
            table_buf.insert(Point2::new(0, y), process_launcher.get_term_view());

            process_launcher.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![ 0 ]
            });
/*
            let mut last_box = Arc::new(RwLock::new(AsciiBox{
                
            }));
*/
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
                        //process_launcher.dn();
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

                        let box_port = ViewPort::new();
                        let test_box = Arc::new(RwLock::new(AsciiBox {
                            content: Some(output_view.map_item(|_,a| a.add_style_back(TerminalStyle::fg_color((230, 230, 230)))).get_view().unwrap()),
                            extent: Vector2::new(120,30)
                        }));

                        box_port.inner().set_view(Some(test_box.clone() as Arc<dyn TerminalView>));

                        table_buf.insert(Point2::new(0, y-1), ViewPort::new().outer());
                        y += 1;
                        table_buf.insert(Point2::new(0, y), box_port.outer()
                                                         .map_item(|_idx, x| x.add_style_back(TerminalStyle::fg_color((90, 120, 100))))
                                                         .offset(Vector2::new(0, -1)));

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

