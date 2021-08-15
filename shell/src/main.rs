
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
        integer::{RadixProjection},
        terminal::{
            Terminal,
            TerminalStyle,
            TerminalAtom,
            TerminalCompositor,
            TerminalEvent,
            make_label,
            TerminalView,
            TerminalEditor},
        string_editor::StringEditor,
        list::{SExprView, ListEditor}
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

    let mut args = std::env::args();
/*
    let arg0_port = ViewPort::new();
    let _arg0 = VecBuffer::<char>::with_data(
        args.next().expect("Arg $0 missing!")
            .chars().collect::<Vec<char>>(),
        arg0_port.inner()
    );
*/
    let arg1_vec_port = ViewPort::new();
    let mut arg1 = VecBuffer::<char>::with_data(
        vec!['1'],
/*
        args.next().expect("Arg $1 missing!")
            .chars().collect::<Vec<char>>(),
*/
        arg1_vec_port.inner()
    );
/*
    let _arg1_vec = args.next().expect("Arg $1 missing!")
            .chars().collect::<Vec<char>>();
*/
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut ed = StringEditor::new();

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    async_std::task::spawn(
        async move {
            let mut ctx = Context::new();
            for tn in vec![
                "MachineWord", "MachineInt", "MachineSyllab", "Bits",
                "Vec", "Stream", "Json",
                "Sequence", "UTF-8-String", "UnicodeChar",
                "PositionalInt", "Digit", "LittleEndian", "BigEndian",
                "DiffStream", "ℕ"
            ] { ctx.add_typename(tn.into()); }

            let src_type =
                ctx.type_term_from_str("( Vec UnicodeChar )").unwrap();

            let dst_type =
                ctx.type_term_from_str("( Sequence UnicodeChar )").unwrap();

            ctx.add_morphism(
                MorphismType {
                    mode: MorphismMode::Epi,
                    src_type: src_type.clone(),
                    dst_type: dst_type.clone()
                },
                Box::new(move |src| {
                    assert!(src.type_tag == src_type);
                    Object {
                        type_tag: dst_type.clone(),
                        repr: ReprTree::new_leaf(
                            src.get_port::<RwLock<Vec<char>>>().unwrap()
                                .to_sequence()
                                .into()
                        )
                    }
                })
            );

            let src_type = ctx.type_term_from_str("( Sequence UnicodeChar )").unwrap();
            let dst_type = ctx.type_term_from_str("( Sequence ( Bits 32 ) )").unwrap();
            ctx.add_morphism(
                MorphismType {
                    mode: MorphismMode::Mono,
                    src_type: src_type.clone(),
                    dst_type: dst_type.clone()
                },
                Box::new({
                    move |src| {
                        assert!(src.type_tag == src_type);
                        Object {
                            type_tag: dst_type.clone(),
                            repr: ReprTree::new_leaf(
                                src.get_port::<dyn SequenceView<Item = char>>().unwrap()
                                    .map(
                                        |c| *c as u32
                                    )
                                    .into()
                            )
                        }
                    }
                })
            );

/*
            let src_type = vec![
                ctx.type_term_from_str("( PositionalInteger  )").unwrap(),
            ];
            let dst_type = ctx.type_term_from_str("( Sequence MachineInt )").unwrap();
            ctx.add_morphism(
                MorphismType {
                    mode: MorphismMode::Epi,
                    src_type: src_type.clone(),
                    dst_type: dst_type.clone()
                },
                Box::new({
                    move |src| {
                        assert!(src.type_tag == src_type);
                        Object {
                            type_tag: dst_type.clone(),
                            repr: ReprTree::new_leaf(
                                vec![ dst_type.clone() ].into_iter(),
                                src.get_port::<RwLock<Vec<usize>>>().unwrap().to_sequence().into()                                
                            )
                        }
                    }
                })
            );
             */
 
            let arg1_vec_port = ed.get_data_port();

            ctx.add_obj("$1".into(), "( Vec UnicodeChar )");
            ctx.insert_repr(
                "$1",
                vec![].into_iter(),
                arg1_vec_port.clone().into()
            );

            ctx.epi_cast("$1", "( Sequence UnicodeChar )");
            ctx.epi_cast("$1", "( Sequence ( Digit 10 ) )");
            ctx.epi_cast("$1", "( PositionalInt 10 LittleEndian )");
            ctx.epi_cast("$1", "( ℕ )");

            let arg1_dec_unic_port: OuterViewPort<dyn SequenceView<Item = char>> =
                ctx.mono_view(
                    "$1",
                    vec![
                        "( PositionalInt 10 LittleEndian )",
                        "( Sequence ( Digit 10 ) )",
                        "( Sequence UnicodeChar )"
                    ].into_iter()
                ).unwrap();

            let arg1_dec_mint_port: OuterViewPort<dyn SequenceView<Item = usize>> =
                arg1_dec_unic_port
                .map(|c| c.to_digit(10).map(|x| x as usize))
                .filter(|d| d.is_some())
                .map(|d| d.unwrap());

            ctx.insert_repr(
                "$1",
                vec![
                    "( PositionalInt 10 LittleEndian )",
                    "( Sequence ( Digit 10 ) )",
                    "( Sequence MachineInt )"
                ].into_iter(),
                arg1_dec_mint_port.clone().into()
            );

            let arg1_hex_mint_port: ViewPort<RwLock<Vec<usize>>>
                = ViewPort::new();
            let _radix_proj = RadixProjection::new(
                10,
                16,
                arg1_dec_mint_port.clone(),
                arg1_hex_mint_port.inner()
            );

            ctx.insert_repr(
                "$1",
                vec![
                    "( PositionalInt 16 LittleEndian )",
                    "( Sequence ( Digit 16 ) )",
                    "( Sequence MachineInt )"
                ].into_iter(),
                arg1_hex_mint_port.outer().to_sequence().into()
            );

            let arg1_hex_unic_port: OuterViewPort<dyn SequenceView<Item = char>> =
                arg1_hex_mint_port.outer().to_sequence()
                .map(
                    |d| char::from_digit(*d as u32, 16).unwrap()
                );

            ctx.insert_repr(
                "$1",
                vec![
                    "( PositionalInt 16 LittleEndian )",
                    "( Sequence ( Digit 16 ) )",
                    "( Sequence UnicodeChar )"
                ].into_iter(),
                arg1_hex_unic_port.clone().into()
            );

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

            {
//                let history_port = ViewPort::new();
//                let mut history = VecBuffer::new(history_port.inner());
/*
                compositor.write().unwrap().push(
                    history_port.into_outer()
                        .to_sequence()
                        .map(
                            |
                        )
                        .to_grid_vertical()
                        .flatten()
                        .offset(Vector2::new(45, 5))
                );
*/
            };
            let mut y = 5;


            // TypeEditor

            let make_sub_editor = || {
                std::sync::Arc::new(std::sync::RwLock::new(StringEditor::new()))
            };

            let mut te = ListEditor::new(make_sub_editor.clone());

            compositor.write().unwrap().push(
                te.path_view()
                    .offset(cgmath::Vector2::new(40,y))
            );
            y += 1;

            let mut p = te.get_data_port().map(|string_editor| string_editor.read().unwrap().get_data_port());

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
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
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

                        te.handle_terminal_event(
                            &TerminalEvent::Input(Event::Key(Key::Up))
                        );
                        te.handle_terminal_event(
                            &TerminalEvent::Input(Event::Key(Key::Home))
                        );
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
                    },
                    ev => {
                        te.handle_terminal_event(&ev);
                    }
                    _ => {}
                }
            }

            //drop(term);
        }
    );

    term_writer.show().await.expect("output error!");
}

