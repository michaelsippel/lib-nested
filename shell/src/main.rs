
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
        tree_nav::{TreeNav, TreeNavResult, TerminalTreeEditor},
        list::{SExprView, ListEditor, ListEditorStyle}
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

            {
                compositor.write().unwrap().push(magic.offset(Vector2::new(40, 4)));
                //compositor.write().unwrap().push(magic.offset(Vector2::new(40, 20)));

                let monstera_port = monstera::make_monstera();
                compositor.write().unwrap().push(monstera_port.clone());
                compositor.write().unwrap().push(monstera_port.offset(Vector2::new(83,0)));

            }

            let cur_size_port = ViewPort::new();
            let mut cur_size = nested::singleton::SingletonBuffer::new(Vector2::new(10, 10), cur_size_port.inner());

            let mut y = 5;

            // TypeEditor

            let make_char_editor = || {
                std::sync::Arc::new(std::sync::RwLock::new(DigitEditor::new(16)))
            };

            let make_sub_editor = move || {
                std::sync::Arc::new(std::sync::RwLock::new(ListEditor::new(make_char_editor.clone(), ListEditorStyle::Hex)))
            };

            let mut te = ListEditor::new(make_sub_editor.clone(), ListEditorStyle::Clist);

            compositor.write().unwrap().push(
                te.get_term_view()
                    .offset(cgmath::Vector2::new(40,y))
            );
            y += 1;

            let mut p = te.get_data_port().map(|sub_editor| sub_editor.read().unwrap().get_data_port());

            let status_chars_port = ViewPort::new();
            let mut status_chars = VecBuffer::new(status_chars_port.inner());

            compositor.write().unwrap().push(
                status_chars_port.outer()
                    .to_sequence()
                    .to_grid_horizontal()
                    .offset(cgmath::Vector2::new(40, 2))
            );

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
                    TerminalEvent::Input(Event::Key(Key::Down)) => { te.dn(); }
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
                        te.handle_terminal_event(&ev);
                    }
                    _ => {}
                }

                status_chars.clear();
                match te.get_cursor() {
                    Some(addr) => {
                        status_chars.push(TerminalAtom::new('@', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                        for x in addr {
                            for c in format!("{}", x).chars() {
                                status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((0, 100, 20))));
                            }
                            status_chars.push(TerminalAtom::new('.', TerminalStyle::fg_color((120, 80, 80))));
                        }
                    }
                    None => {}
                }
            }

            //drop(term);
        }
    );

    term_writer.show().await.expect("output error!");
}

