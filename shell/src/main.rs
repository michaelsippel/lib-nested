use {
    std::sync::RwLock,
    cgmath::Point2,    
    termion::event::{Event, Key},
    nested::{
        core::{
            ViewPort,
            OuterViewPort,
            context::{
                ReprTree,
                Object,
                MorphismType,
                MorphismMode,
                Context
            },
            port::{
                UpdateTask
            }
        },
        sequence::{SequenceView, VecBuffer},
        integer::{RadixProjection},
        terminal::{Terminal, TerminalAtom, TerminalStyle, TerminalCompositor, TerminalEvent},
        string_editor::StringEditor
    }
};

pub mod list;

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

    let arg0_port = ViewPort::new();
    let _arg0 = VecBuffer::<char>::with_data(
        args.next().expect("Arg $0 missing!")
            .chars().collect::<Vec<char>>(),
        arg0_port.inner()
    );
/*
    let arg1_vec_port = ViewPort::new();
    let mut arg1 = VecBuffer::<char>::with_data(
        args.next().expect("Arg $1 missing!")
            .chars().collect::<Vec<char>>(),
        arg1_vec_port.inner()
    );
*/

    let arg1_vec = args.next().expect("Arg $1 missing!")
            .chars().collect::<Vec<char>>();

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

            compositor.write().unwrap().push(
                ed.insert_view()
                    .map_key(
                        |pt| Point2::new(40 as i16 - pt.x, 1 as i16),
                        |pt| if pt.y == 1 { Some(Point2::new(40 as i16 - pt.x, 0)) } else { None }
                    )
                    .map_item(
                        |_pos, atom|
                        TerminalAtom::new(
                            atom.c.unwrap_or(' '),
                            TerminalStyle::fg_color(
                                if let Some(c) = atom.c {
                                    if c == '|' {
                                        (200, 200, 90)
                                    } else if c.is_digit(10) {
                                        (0, 200, 0)
                                    } else {
                                        (255, 0, 0)
                                    }
                                } else {
                                    (0, 0, 0)
                                }
                            )
                        )
                    )
            );

            let opening_port = ViewPort::new();
            let opening = VecBuffer::<char>::with_data("]".chars().collect(), opening_port.inner());

            let dec_label_port = ViewPort::new();
            let dec_label = VecBuffer::<char>::with_data("d0".chars().collect(), dec_label_port.inner());

            let hex_label_port = ViewPort::new();
            let hex_label = VecBuffer::<char>::with_data("x0".chars().collect(), hex_label_port.inner());
            
            let delim_port = ViewPort::new();
            let delim = VecBuffer::<char>::with_data(",".chars().collect(), delim_port.inner());

            let closing_port = ViewPort::new();
            let closing = VecBuffer::<char>::with_data("[".chars().collect(), closing_port.inner());
            for c in arg1_vec {
                ed.insert(c);
                ed.prev();
            }

            {
                let tree_port = ViewPort::new();
                let mut tree = VecBuffer::with_data(
                    vec![
                        opening_port.outer()
                            .to_sequence()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((170, 170, 30)))),

                        arg1_dec_mint_port
                            .map(|val| char::from_digit(*val as u32, 16).unwrap())
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((255, 255, 255)))),

                        dec_label_port.outer()
                            .to_sequence()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((170, 170, 170)))),

                        delim_port.outer()
                            .to_sequence()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((170, 170, 30)))),

                        arg1_hex_unic_port.clone()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((255, 255, 255)))),

                        hex_label_port.outer()
                            .to_sequence()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((170, 170, 170)))),

                        closing_port.outer()
                            .to_sequence()
                            .map(|c| TerminalAtom::new(*c, TerminalStyle::fg_color((170, 170, 30)))),
                    ],
                    tree_port.inner()
                );

                compositor.write().unwrap().push(
                    tree_port.outer()
                        .to_sequence()
                        .flatten()
                        .to_index()
                        .map_key(
                            |idx| Point2::new(40 - *idx as i16, 2 as i16),
                            |pt| if pt.y == 2 { Some(40 - pt.x as usize) } else { None }
                        )
                );
            }

            {
                let items_port = ViewPort::new();
                let items = VecBuffer::with_data(
                    vec![
                        arg1_dec_mint_port
                            .map(|val| char::from_digit(*val as u32, 16).unwrap())
                            .map(|c| TerminalAtom::from(c)),
                        arg1_hex_unic_port.clone()
                            .map(|c| TerminalAtom::from(c)),
                        arg1_hex_unic_port.clone()
                            .map(|c| TerminalAtom::from(c)),
                    ],
                    items_port.inner()
                );

                let liport = ViewPort::new();
                let list_decorator = list::ListDecorator::lisp_style(
                    1,
                    items_port.outer().to_sequence(),
                    liport.inner()
                );

                let par_items_port = ViewPort::new();
                let par_items = VecBuffer::with_data(
                    vec![
                        liport.outer().flatten(),
                        arg1_hex_unic_port.clone()
                            .map(|c| TerminalAtom::from(c)),
                    ],
                    par_items_port.inner()
                );

                let par_liport = ViewPort::new();
                let par_list_decorator = list::ListDecorator::lisp_style(
                    0,
                    par_items_port.outer().to_sequence(),
                    par_liport.inner()
                );

                compositor.write().unwrap().push(
                    par_liport.outer()
                        .flatten()
                        .to_index()
                        .map_key(
                            |idx| Point2::new(*idx as i16, 3),
                            |pt| if pt.y == 3 { Some(pt.x as usize) } else { None }
                        )
                );
            }

            let magic_vec_port = ViewPort::new();
            let _magic_vec = VecBuffer::with_data("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>".chars().collect::<Vec<char>>(), magic_vec_port.inner());

            compositor.write().unwrap().push(
                magic_vec_port.outer()
                    .to_sequence()
                    .to_index()
                    .map_item(
                        |idx, c| TerminalAtom::new(
                            *c,
                            TerminalStyle::fg_color((5, ((80+(idx*30)%100) as u8), (55+(idx*15)%180) as u8))
                        )
                    )
                    .map_key(
                        |idx| Point2::new(*idx as i16, 4),
                        |pt| if pt.y == 4 { Some(pt.x as usize) } else { None }
                    )
            );

            compositor.write().unwrap().push(
                magic_vec_port.outer()
                    .to_sequence()
                    .to_index()
                    .map_item(
                        |idx, c| TerminalAtom::new(
                            *c,
                            TerminalStyle::fg_color((5, ((80+(idx*30)%100) as u8), (55+(idx*15)%180) as u8))
                        )
                    )
                    .map_key(
                        |idx| Point2::new(*idx as i16, 0),
                        |pt| if pt.y == 0 { Some(pt.x as usize) } else { None }
                    )
            );

            term_port.update();

            loop {
                match term.next_event().await {
                    TerminalEvent::Input(Event::Key(Key::Left)) => ed.next(),
                    TerminalEvent::Input(Event::Key(Key::Right)) => ed.prev(),
                    TerminalEvent::Input(Event::Key(Key::Home)) => ed.goto_end(),
                    TerminalEvent::Input(Event::Key(Key::End)) => ed.goto(0),
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {},
                    TerminalEvent::Input(Event::Key(Key::Char(c))) => { ed.insert(c); ed.prev(); },
                    TerminalEvent::Input(Event::Key(Key::Delete)) => ed.delete_prev(),
                    TerminalEvent::Input(Event::Key(Key::Backspace)) => ed.delete(),
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) => break,
                    _ => {}
                }
                term_port.update();
            }

            drop(term);
        }
    );

    term_writer.show().await.expect("output error!");
}

