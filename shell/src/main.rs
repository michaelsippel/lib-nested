
mod monstera;

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
        sequence::{SequenceView},
        vec::{VecBuffer},
        integer::{RadixProjection, DigitEditor},
        terminal::{
            Terminal,
            TerminalStyle,
            TerminalAtom,
            TerminalCompositor,
            TerminalEvent,
            make_label,
            TerminalView,
            TerminalEditor},
        string_editor::{CharEditor},
        tree_nav::{TreeNav, TreeNavResult, TreeCursor, TerminalTreeEditor},
        list::{SExprView, ListCursorMode, ListEditor, ListEditorStyle}
    }
};

struct GridFill<T: Send + Sync + Clone>(T);
impl<T: Send + Sync + Clone> View for GridFill<T> {
    type Msg = Point2<i16>;
}

impl<T: Send + Sync + Clone> IndexView<Point2<i16>> for GridFill<T> {
    type Item = T;

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        None
    }

    fn get(&self, _: &Point2<i16>) -> Option<T> {
        Some(self.0.clone())
    }
}

#[async_std::main]
async fn main() {
    /* todo:

open::
>0:
( Path )
( Sequence ( Sequence UnicodeChar ) )
( Sequence UnicodeChar )
<1:
( FileDescriptor )
( MachineInt )

read::
>0:
( FileDescriptor )
( MachineInt )
<1:
( Sequence MachineSyllab )
( Vec MachineSyllab )

write::
>0
( FileDescriptor )
( MachineInt )
>1:
( Sequence MachineSyllab )
( Vec MachineSyllab )

    */

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

            // TypeEditor
            let make_char_editor = || {
                std::sync::Arc::new(std::sync::RwLock::new(CharEditor::new()))
            };
            let make_subsub_editor = move || {
                std::sync::Arc::new(std::sync::RwLock::new(ListEditor::new(make_char_editor.clone(), ListEditorStyle::String)))
            };
            let make_sub_editor = move || {
                std::sync::Arc::new(std::sync::RwLock::new(ListEditor::new(make_subsub_editor.clone(), ListEditorStyle::HorizontalSexpr)))
            };

            let mut te = ListEditor::new(make_sub_editor.clone(), ListEditorStyle::VerticalSexpr);

            te.goto(
                TreeCursor {
                    leaf_mode: ListCursorMode::Insert,
                    tree_addr: vec![ 0 ]
                }
            );

            let mut p = te.get_data_port().map(|sub_editor| sub_editor.read().unwrap().get_data_port());

            let status_chars_port = ViewPort::new();
            let mut status_chars = VecBuffer::new(status_chars_port.inner());

            let help_port = ViewPort::<dyn nested::grid::GridView<Item = OuterViewPort<dyn TerminalView>>>::new();
            let mut help_buf = nested::index::buffer::IndexBuffer::<Point2<i16>, OuterViewPort<dyn TerminalView>>::new(help_port.inner());

            let table_style = TerminalStyle::fg_color((120, 100, 80));
            let desc_style = TerminalStyle::italic(true);
            help_buf.insert_iter(vec![
                (Point2::new(0, 0), make_label("CTRL+{c,d,g}").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 0), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 0), make_label("quit").map_item(move |_idx, atom| atom.add_style_back(desc_style))),

                (Point2::new(0, 1), make_label("↞ ← ↑ ↓ → ↠").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 1), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 1), make_label("move cursor").map_item(move |_idx, atom| atom.add_style_back(desc_style))),

                (Point2::new(0, 3), make_label("<DEL> (Select)").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 3), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 3), make_label("delete item at cursor position").map_item(move |_idx, atom| atom.add_style_back(desc_style))),

                (Point2::new(0, 4), make_label("<DEL> (Insert)").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 4), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 4), make_label("delete item right to cursor").map_item(move |_idx, atom| atom.add_style_back(desc_style))),

                (Point2::new(0, 5), make_label("<BACKSPACE> (Insert)").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 5), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 5), make_label("delete item left to cursor").map_item(move |_idx, atom| atom.add_style_back(desc_style))),

                (Point2::new(0, 6), make_label("<TAB>").map_item(|_idx, atom| atom.add_style_back(TerminalStyle::bold(true)))),
                (Point2::new(1, 6), make_label(" | ").map_item(move |_idx, atom| atom.add_style_back(table_style))),
                (Point2::new(2, 6), make_label("toggle cursor mode (insert / select)").map_item(move |_idx, atom| atom.add_style_back(desc_style))),
            ]);

            let help_head = make_label("─────────────────────┬─────────────────────").map_item(move |_idx, atom| atom.add_style_back(table_style));

            table_buf.insert_iter(vec![
                (Point2::new(0, 0), magic.clone()),
                (Point2::new(0, 2), status_chars_port.outer().to_sequence().to_grid_horizontal()),
                (Point2::new(0, 3), te.get_term_view()),
                (Point2::new(0, 4), make_label(" ")),
                (Point2::new(0, 5), help_head),
                (Point2::new(0, 6), help_port.outer().flatten()),
                (Point2::new(0, 7), magic.clone()),
            ]);

            compositor.write().unwrap().push(monstera::make_monstera());
            compositor.write().unwrap().push(table_port.outer().flatten().offset(Vector2::new(40, 2)));

/*
            te.get_data_port()
                .map(
                    |item_editor| item_editor.read().unwrap().get_data_port()
                )
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
                        if te.pxev() == TreeNavResult::Exit {
                            te.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        if te.nexd() == TreeNavResult::Exit {
                            te.goto_end();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => { te.up(); }
                    TerminalEvent::Input(Event::Key(Key::Down)) => { te.dn(); te.goto_home(); }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        if te.goto_home() == TreeNavResult::Exit {

                            te.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        if te.goto_end() == TreeNavResult::Exit {
                            te.goto_end();
                        }
                    }

                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        /*
                        let mut strings = Vec::new();

                        let v = p.get_view().unwrap();
                        for i in 0 .. v.len().unwrap_or(0) {
                            strings.push(
                                v
                                    .get(&i).unwrap()
                                    .get_view().unwrap()
                                    .read().unwrap()
                                    .iter().collect::<String>()
                            );
                        }

                        if strings.len() == 0 { continue; }
                        
                        if let Ok(output) =
                            std::process::Command::new(strings[0].as_str())
                            .args(&strings[1..])
                            .output()
                        {
                            // take output and update terminal view
                            let mut line_port = ViewPort::new();
                            let mut line = VecBuffer::new(line_port.inner());
                            for byte in output.stdout {
                                match byte {
                                    b'\n' => {
                                        compositor.write().unwrap().push(
                                            line_port.outer()
                                                .to_sequence()
                                                .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((130,90,90))))
                                                .to_grid_horizontal()
                                                .offset(Vector2::new(45, y))
                                        );
                                        y += 1;
                                        line_port = ViewPort::new();
                                        line = VecBuffer::new(line_port.inner());
                                    }
                                    byte => {
                                        line.push(byte as char);
                                    }
                                }
                            }
                        } else {
                            compositor.write().unwrap().push(
                                make_label("Command not found")
                                    .map_item(|idx, a| a.add_style_back(TerminalStyle::fg_color((200,0,0))))
                                    .offset(Vector2::new(45, y))
                            );
                            y+=1;
                        }

                        te.up();
                        te.goto_home();
                        te = ListEditor::new(make_sub_editor.clone());

                        compositor.write().unwrap().push(magic.offset(Vector2::new(40, y)));
                        y += 1;
                        compositor.write().unwrap().push(
                            te
                                .horizontal_sexpr_view()
                                .offset(Vector2::new(40, y))
                        );
                        y += 1;

                        p = te.get_data_port().map(|string_editor| string_editor.read().unwrap().get_data_port());
*/
                    },
                    ev => {
                        if te.get_cursor().leaf_mode == ListCursorMode::Select {
                            match ev {
                                TerminalEvent::Input(Event::Key(Key::Char('l'))) => { te.up(); },
                                TerminalEvent::Input(Event::Key(Key::Char('a'))) => { te.dn(); },
                                TerminalEvent::Input(Event::Key(Key::Char('i'))) => { te.pxev(); },
                                TerminalEvent::Input(Event::Key(Key::Char('e'))) => { te.nexd(); },
                                TerminalEvent::Input(Event::Key(Key::Char('u'))) => { te.goto_home(); },
                                TerminalEvent::Input(Event::Key(Key::Char('o'))) => { te.goto_end(); },
                                _ => {
                                    te.handle_terminal_event(&ev);
                                }
                            }
                        } else {
                            te.handle_terminal_event(&ev);
                        }
                    }
                }

                status_chars.clear();
                let cur = te.get_cursor();

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
                }
            }

            //drop(term);
        }
    );

    term_writer.show().await.expect("output error!");
}

