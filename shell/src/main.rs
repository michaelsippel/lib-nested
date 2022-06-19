extern crate portable_pty;

mod ascii_box;
mod monstera;
mod process;
mod pty;

mod plot;

use {
    crate::process::ProcessLauncher,
    cgmath::{Point2, Vector2},
    nested::{
        core::{port::UpdateTask, Observer, OuterViewPort, ViewPort},
        index::IndexArea,
        list::{ListCursorMode, PTYListEditor},
        sequence::{decorator::{SeqDecorStyle}},
        terminal::{
            make_label, Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView,
        },
        tree_nav::{TreeNav, TerminalTreeEditor, TreeCursor, TreeNavResult},
        vec::VecBuffer,
    },
    std::sync::{Arc, RwLock},
    termion::event::{Event, Key},
};

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    async_std::task::spawn(async move {
        let mut table = nested::index::buffer::IndexBuffer::new();

        let magic =
            make_label("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>").map_item(|pos, atom| {
                atom.add_style_back(TerminalStyle::fg_color((
                    5,
                    ((80 + (pos.x * 30) % 100) as u8),
                    (55 + (pos.x * 15) % 180) as u8,
                )))
            });

        let mut cur_size = nested::singleton::SingletonBuffer::new(Vector2::new(10, 10));
        let mut status_chars = VecBuffer::new();

        let mut process_list_editor = PTYListEditor::new(
            Box::new(|| Arc::new(RwLock::new(ProcessLauncher::new()))),
            SeqDecorStyle::VerticalSexpr,
            0
        );

        let mut plist = VecBuffer::new();
        let mut plist_port = plist.get_port();
        async_std::task::spawn(async move {
            let (w, _h) = termion::terminal_size().unwrap();
            let mut x: usize = 0;
            loop {
                let val = (5.0
                    + (x as f32 / 3.0).sin() * 5.0
                    + 2.0
                    + ((7 + x) as f32 / 5.0).sin() * 2.0
                    + 2.0
                    + ((9 + x) as f32 / 10.0).cos() * 3.0) as usize;

                if x < w as usize {
                    plist.push(val);
                } else {
                    *plist.get_mut(x % (w as usize)) = val;
                }

                x += 1;
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;

                if x % (w as usize) == 0 {
                    async_std::task::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        });

        let plot_port = ViewPort::new();
        let _plot = crate::plot::Plot::new(plist_port.to_sequence(), plot_port.inner());

        table.insert_iter(vec![
            (Point2::new(0, 0), magic.clone()),
            (
                Point2::new(0, 1),
                status_chars.get_port().to_sequence().to_grid_horizontal(),
            ),
            (Point2::new(0, 2), magic.clone()),
            (Point2::new(0, 3), process_list_editor.get_term_view()),
        ]);

        let (w, h) = termion::terminal_size().unwrap();

        compositor.write().unwrap().push(
            plot_port.outer()
                .map_item(|pt, a| {
                    a.add_style_back(TerminalStyle::fg_color((
                        255 - pt.y as u8 * 8,
                        100,
                        pt.y as u8 * 15,
                    )))
                })
                .offset(Vector2::new(0, h as i16 - 20)),
        );

        compositor
            .write()
            .unwrap()
            .push(monstera::make_monstera().offset(Vector2::new(w as i16 - 38, 0)));

        compositor
            .write()
            .unwrap()
            .push(table.get_port().flatten().offset(Vector2::new(3, 0)));

        process_list_editor.goto(TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        });

        let tp = term_port.clone();
        async_std::task::spawn(async move {
            loop {
                tp.update();
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;
            }
        });

        loop {
            status_chars.clear();
            let cur = process_list_editor.get_cursor();

            if cur.tree_addr.len() > 0 {
                status_chars.push(TerminalAtom::new(
                    '@',
                    TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true)),
                ));
                for x in cur.tree_addr {
                    for c in format!("{}", x).chars() {
                        status_chars
                            .push(TerminalAtom::new(c, TerminalStyle::fg_color((0, 100, 20))));
                    }
                    status_chars.push(TerminalAtom::new(
                        '.',
                        TerminalStyle::fg_color((120, 80, 80)),
                    ));
                }

                status_chars.push(TerminalAtom::new(
                    ':',
                    TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true)),
                ));
                for c in match cur.leaf_mode {
                    ListCursorMode::Insert => "INSERT",
                    ListCursorMode::Select => "SELECT"
                }
                .chars()
                {
                    status_chars.push(TerminalAtom::new(
                        c,
                        TerminalStyle::fg_color((200, 200, 20)),
                    ));
                }
                status_chars.push(TerminalAtom::new(
                    ':',
                    TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true)),
                ));
            } else {
                for c in "Press <DN> to enter".chars() {
                    status_chars.push(TerminalAtom::new(
                        c,
                        TerminalStyle::fg_color((200, 200, 20)),
                    ));
                }
            }

            let ev = term.next_event().await;

            if let TerminalEvent::Resize(new_size) = ev {
                cur_size.set(new_size);
                term_port.inner().get_broadcast().notify(&IndexArea::Full);
                continue;
            }

            if let Some(process_editor) = process_list_editor.get_item() {
                let mut pe = process_editor.write().unwrap();
                if pe.is_captured() {
                    if let TerminalEditorResult::Exit = pe.handle_terminal_event(&ev) {
                        drop(pe);
                        process_list_editor.up();
                        process_list_editor.nexd();
                    }
                    continue;
                }
            }

            match ev {
                TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,
                TerminalEvent::Input(Event::Key(Key::Ctrl('l'))) => {
                    process_list_editor.goto(TreeCursor {
                        leaf_mode: ListCursorMode::Insert,
                        tree_addr: vec![0],
                    });
                    process_list_editor.clear();
                }
                TerminalEvent::Input(Event::Key(Key::Left)) => {
                    process_list_editor.pxev();
                }
                TerminalEvent::Input(Event::Key(Key::Right)) => {
                    process_list_editor.nexd();
                }
                TerminalEvent::Input(Event::Key(Key::Up)) => {
                    if process_list_editor.up() == TreeNavResult::Exit {
                        process_list_editor.dn();
                    }
                }
                TerminalEvent::Input(Event::Key(Key::Down)) => {
                    process_list_editor.dn();
                    // == TreeNavResult::Continue {
                        //process_list_editor.goto_home();
                    //}
                }
                TerminalEvent::Input(Event::Key(Key::Home)) => {
                    process_list_editor.qpxev();
                }
                TerminalEvent::Input(Event::Key(Key::End)) => {
                    process_list_editor.qnexd();
                }
                ev => {
                    if process_list_editor.get_cursor().leaf_mode == ListCursorMode::Select {
                        match ev {
                            TerminalEvent::Input(Event::Key(Key::Char('l'))) => {
                                process_list_editor.up();
                            }
                            TerminalEvent::Input(Event::Key(Key::Char('a'))) => {
                                process_list_editor.dn();
                            }
                            TerminalEvent::Input(Event::Key(Key::Char('i'))) => {
                                process_list_editor.pxev();
                            }
                            TerminalEvent::Input(Event::Key(Key::Char('e'))) => {
                                process_list_editor.nexd();
                            }
                            TerminalEvent::Input(Event::Key(Key::Char('u'))) => {
                                process_list_editor.qpxev();
                            }
                            TerminalEvent::Input(Event::Key(Key::Char('o'))) => {
                                process_list_editor.qnexd();
                            }
                            _ => {
                                process_list_editor.handle_terminal_event(&ev);
                            }
                        }
                    } else {
                        if let TerminalEditorResult::Exit =
                            process_list_editor.handle_terminal_event(&ev)
                        {
                            process_list_editor.nexd();
                        }
                    }
                }
            }
        }

        drop(term);
        drop(term_port);
    });

    term_writer.show().await.expect("output error!");
}
