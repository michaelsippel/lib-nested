extern crate portable_pty;

mod pty;
mod incubator;

// TODO rewrite process & command with incubator rules
//mod process;
//mod command;

use {
    cgmath::{Point2, Vector2},
    nested::{
        core::{port::UpdateTask, Observer, AnyOuterViewPort, ViewPort},
        type_system::{Context, ReprTree},
        index::IndexArea,
        list::{ListCursorMode, PTYListEditor},
        sequence::{decorator::{SeqDecorStyle, Separate}},
        terminal::{
            make_label, Terminal, TerminalAtom, TerminalCompositor, TerminalEditor,
            TerminalEditorResult, TerminalEvent, TerminalStyle,
        },
        tree::{TreeNav, TreeCursor, TreeNavResult},
        vec::VecBuffer,
        diagnostics::{Diagnostics},
        index::{buffer::IndexBuffer},
        commander::Commander
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

    let portmutex = Arc::new(RwLock::new(()));

    // Update Loop //
    let tp = term_port.clone();
    async_std::task::spawn({
        let portmutex = portmutex.clone();
        async move {
            loop {
                {
                    let _l = portmutex.write().unwrap();
                    tp.update();
                }
                async_std::task::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    });

    // Type Context //
    let ctx = Arc::new(RwLock::new(Context::new()));
    let ctx = nested::type_system::init_mem_ctx(ctx);
    let ctx = nested::type_system::init_editor_ctx(ctx);
    let ctx = nested::type_system::init_math_ctx(ctx);
    let ctx = nested::type_system::init_os_ctx(ctx);

    let vb = VecBuffer::<char>::new();
    let rt_char = ReprTree::new_leaf(
        ctx.read().unwrap().type_term_from_str("( Vec Char )").unwrap(),
        AnyOuterViewPort::from(vb.get_port())
    );

    let rt_digit = ReprTree::ascend(&rt_char, ctx.read().unwrap().type_term_from_str("( List ( Digit 10 ) )").unwrap());
    rt_digit.write().unwrap().insert_branch(
        ReprTree::new_leaf(
            ctx.read().unwrap().type_term_from_str("( Vec MachineInt )").unwrap(),
            AnyOuterViewPort::from(
                vb.get_port().to_sequence().map(
                    |c: &char| {
                        c.to_digit(10).unwrap()
                    }
                )
            )
        )
    );

/*    
    ctx.write().unwrap().add_morphism(
        MorphismType{
            mode: MorphismMode::Iso,
            src_type: 
        },
        Box::new(
            |repr| {
                RadixProjection::new(
                    
                )
            }
        )
    );
     */

    let c = ctx.clone();
    let mut process_list_editor =
        PTYListEditor::new(
            ctx.clone(),
            c.read().unwrap().type_term_from_str("( List Path 1 )").unwrap(),
            SeqDecorStyle::Plain,
            Some('\n'),
            3
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
             process_list_editor
             .editor.read().unwrap()
             .get_seg_seq_view()
             .enumerate()
             .map(
                 |(n, segment)| {
                     let mut buf = IndexBuffer::new();
                     buf.insert_iter(vec![
                         (Point2::new(0, 0),
                          make_label(match n+1 {
                              1 => "I) ",
                              2 => "II) ",
                              3 => "III) ",
                              4 => "IV) ",
                              5 => "V) ",
                              6 => "VI) ",
                              7 => "VII) ",
                              8 => "IIX) ",
                              9 => "IX) ",
                              10 => "X) ",
                              _ => ""
                          })),
                         (Point2::new(1, 0), segment.clone())
                     ]);

                     buf.get_port()
                 }
             )
             .separate({
                 let mut buf = IndexBuffer::new();
                 buf.insert(Point2::new(1,0),
                            make_label(" ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~  ~~")
                            .map_item(
                                |p,a|
                                a.add_style_front(TerminalStyle::fg_color((40,40,40)))
                            )
                 );
                 buf.get_port()
             })
             .to_grid_vertical()
             .flatten()
             .flatten()
            ),

            (Point2::new(0, 5), make_label(" ")),
            (Point2::new(0, 6), magic.clone()),

            (Point2::new(0, 7), process_list_editor.diag.map(
                |entry| {
                    let mut b = VecBuffer::new();
                    b.push(
                         make_label("@")
                         .map_item(|_p,a| a
                                   .add_style_back(TerminalStyle::bold(true))
                                   .add_style_back(TerminalStyle::fg_color((120,120,0))))
                    );

                    for x in entry.addr.iter() {
                        b.push(
                            make_label(&format!("{}", x))
                                .map_item(|_p,a| a
                                          .add_style_back(TerminalStyle::fg_color((0, 100, 20))))
                        );
                        b.push(
                            make_label(".")
                                .map_item(|_p,a| a
                                   .add_style_back(TerminalStyle::bold(true))
                                   .add_style_back(TerminalStyle::fg_color((120,120,0))))
                        );
                    }

                    b.push(entry.port.clone());
                    b.get_port()
                        .to_sequence()
                        .to_grid_horizontal()
                        .flatten()
                        .map_item(move |_p,a| {
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

        let (_w, _h) = termion::terminal_size().unwrap();
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

        process_list_editor.editor.write().unwrap().goto(TreeCursor {
            leaf_mode: ListCursorMode::Insert,
            tree_addr: vec![0],
        });

        loop {
            let ev = term.next_event().await;
            let _l = portmutex.write().unwrap();

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
                    process_list_editor.editor.write().unwrap().goto(TreeCursor {
                        leaf_mode: ListCursorMode::Insert,
                        tree_addr: vec![0],
                    });
                    //process_list_editor.clear();
                }
                TerminalEvent::Input(Event::Key(Key::Left)) => {
                    process_list_editor.editor.write().unwrap().pxev();
                }
                TerminalEvent::Input(Event::Key(Key::Right)) => {
                    process_list_editor.editor.write().unwrap().nexd();
                }
                TerminalEvent::Input(Event::Key(Key::Up)) => {
                    if process_list_editor.editor.write().unwrap().up() == TreeNavResult::Exit {
                        process_list_editor.editor.write().unwrap().dn();
                    }
                }
                TerminalEvent::Input(Event::Key(Key::Down)) => {
                    process_list_editor.editor.write().unwrap().dn();
                    // == TreeNavResult::Continue {
                        //process_list_editor.goto_home();
                    //}
                }
                TerminalEvent::Input(Event::Key(Key::Home)) => {
                    process_list_editor.editor.write().unwrap().qpxev();
                }
                TerminalEvent::Input(Event::Key(Key::End)) => {
                    process_list_editor.editor.write().unwrap().qnexd();
                }
                TerminalEvent::Input(Event::Key(Key::Char('\t'))) => {
                    let mut c = process_list_editor.editor.read().unwrap().get_cursor();
                    c.leaf_mode = match c.leaf_mode {
                        ListCursorMode::Select => ListCursorMode::Insert,
                        ListCursorMode::Insert => ListCursorMode::Select
                    };
                    process_list_editor.editor.write().unwrap().goto(c);
                }
                ev => {
                    process_list_editor.send_cmd(&ev);
                }
            }

            status_chars.clear();
            let cur = process_list_editor.editor.read().unwrap().get_cursor();

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
        }

        drop(term);
        drop(term_port);
    });

    term_writer.show().await.expect("output error!");
}

