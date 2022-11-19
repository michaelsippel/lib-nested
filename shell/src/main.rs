extern crate portable_pty;

mod ascii_box;
mod monstera;
mod process;
mod pty;
mod command;
mod plot;

use {
    crate::{
        process::ProcessLauncher,
        command::Commander
    },
    cgmath::{Point2, Vector2},
    nested::{
        core::{port::UpdateTask, Observer, OuterViewPort, View, ViewPort, Context, TypeTerm},
        index::IndexArea,
        list::{ListCursorMode, PTYListEditor},
        sequence::{SequenceView, decorator::{SeqDecorStyle, Separate}},
        terminal::{
            make_label, Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle, TerminalView,
        },
        tree::{TreeNav, TreeCursor, TreeNavResult},
        vec::VecBuffer,
        integer::{PosIntEditor},
        char_editor::CharEditor,
        product::ProductEditor,
        sum::SumEditor,
        diagnostics::{Diagnostics},
        Nested
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

    // Update Loop //
    let tp = term_port.clone();
    async_std::task::spawn(async move {
        loop {
            tp.update();
            async_std::task::sleep(std::time::Duration::from_millis(30)).await;
        }
    });

    // Type Context //
    let ctx = Arc::new(RwLock::new(Context::new()));
    let ctx = nested::make_editor::init_editor_ctx(ctx);
    let ctx = nested::make_editor::init_math_ctx(ctx);
    let ctx = nested::make_editor::init_os_ctx(ctx);

    let c = ctx.clone();
    let mut process_list_editor =
        PTYListEditor::new(
            Box::new( move || {
                Arc::new(RwLock::new(Commander::new(ctx.clone())))
            }),
            SeqDecorStyle::Plain,
            '\0',
            0
        );

    async_std::task::spawn(async move {
        let mut table = nested::index::buffer::IndexBuffer::new();
        
        let magic =
            make_label("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>")
            .map_item(|pos, atom| {
                atom.add_style_back(TerminalStyle::fg_color((
                    5,
                    ((80 + (pos.x * 30) % 100) as u8),
                    (55 + (pos.x * 15) % 180) as u8,
                )))
            });

        let mut cur_size = nested::singleton::SingletonBuffer::new(Vector2::new(10, 10));
        let mut status_chars = VecBuffer::new();

        table.insert_iter(vec![
            (Point2::new(0, 0), magic.clone()),
            (
                Point2::new(0, 1),
                status_chars.get_port().to_sequence().to_grid_horizontal(),
            ),
            (Point2::new(0, 2), magic.clone()),
            (Point2::new(0, 3), make_label(" ")),
            (Point2::new(0, 4),
             process_list_editor.editor
             .get_seg_seq_view()
             .separate(
                 make_label(" ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~")
                     .map_item(|p,a| a.add_style_front(TerminalStyle::fg_color((40,40,40))))
             )
             .to_grid_vertical()
             .flatten()),

            (Point2::new(0, 5), make_label(" ")),
            (Point2::new(0, 6), magic.clone()),

            (Point2::new(0, 7), process_list_editor.get_msg_port().map(
                |entry| {
                    let mut b = VecBuffer::new();
                    b.push(
                         make_label("@")
                         .map_item(|p,a| a
                                   .add_style_back(TerminalStyle::bold(true))
                                   .add_style_back(TerminalStyle::fg_color((120,120,0))))
                    );

                    for x in entry.addr.iter() {
                        b.push(
                            make_label(&format!("{}", x))
                                .map_item(|p,a| a
                                          .add_style_back(TerminalStyle::fg_color((0, 100, 20))))
                        );
                        b.push(
                            make_label(".")
                                .map_item(|p,a| a
                                   .add_style_back(TerminalStyle::bold(true))
                                   .add_style_back(TerminalStyle::fg_color((120,120,0))))
                        );
                    }

                    b.push(entry.port.clone());
                    b.get_port()
                        .to_sequence()
                        .to_grid_horizontal()
                        .flatten()
                        .map_item(move |p,a| {
                            let select = false;
                            if select {
                                a.add_style_back(TerminalStyle::fg_color((60,60,60)))
                            } else {
                                *a
                            }
                        })
                }
            ).to_grid_vertical().flatten())

        ]);

        let (w, h) = termion::terminal_size().unwrap();
/*
        compositor
            .write()
            .unwrap()
        .push(monstera::make_monstera().offset(Vector2::new(w as i16 - 38, 0)));
*/
        compositor
            .write()
            .unwrap()
            .push(table.get_port().flatten().offset(Vector2::new(3, 0)));

        process_list_editor.goto(TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        });

        loop {
            status_chars.clear();
            let cur = process_list_editor.get_cursor();

            if cur.tree_addr.len() > 0 {
                status_chars.push(TerminalAtom::new(
                    '@',
                    TerminalStyle::fg_color((150, 80,230)).add(TerminalStyle::bold(true)),
                ));
                for x in cur.tree_addr {
                    for c in format!("{}", x).chars() {
                        status_chars
                            .push(TerminalAtom::new(c, TerminalStyle::fg_color((0, 100, 20))));
                    }
                    status_chars.push(TerminalAtom::new(
                        '.',
                        TerminalStyle::fg_color((150, 80,230))
                    ));
                }

                status_chars.push(TerminalAtom::new(
                    ':',
                    TerminalStyle::fg_color((150, 80,230)).add(TerminalStyle::bold(true)),
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
                    TerminalStyle::fg_color((150, 80,230)).add(TerminalStyle::bold(true)),

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
/*
            if let Some(process_editor) = process_list_editor.get_item() {
                let mut pe = process_editor.write().unwrap();
                /*
                if pe.is_captured() {
                    if let TerminalEditorResult::Exit = pe.handle_terminal_event(&ev) {
                        drop(pe);
                        process_list_editor.up();
                        process_list_editor.nexd();
                    }
                    continue;
            }
                */
            }
*/            
            match ev {
                TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,
                TerminalEvent::Input(Event::Key(Key::Ctrl('l'))) => {
                    process_list_editor.goto(TreeCursor {
                        leaf_mode: ListCursorMode::Insert,
                        tree_addr: vec![0],
                    });
                    //process_list_editor.clear();
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
                TerminalEvent::Input(Event::Key(Key::Char('\t'))) => {
                    let mut c = process_list_editor.get_cursor();
                    c.leaf_mode = match c.leaf_mode {
                        ListCursorMode::Select => ListCursorMode::Insert,
                        ListCursorMode::Insert => ListCursorMode::Select
                    };
                    process_list_editor.goto(c);
                }
                ev => {
                    if let TerminalEditorResult::Exit =
                        process_list_editor.handle_terminal_event(&ev)
                    {
                        //process_list_editor.nexd();
                    }
                }
            }
        }

        drop(term);
        drop(term_port);
    });

    term_writer.show().await.expect("output error!");
}

