
extern crate portable_pty;

mod monstera;
mod process;
mod pty;
mod ascii_box;

use{
    std::sync::{Arc, RwLock, Mutex},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            InnerViewPort,
            OuterViewPort,
            Observer,
            ObserverExt,
            ObserverBroadcast,
            context::{ReprTree, Object, MorphismType, MorphismMode, Context},
            port::{UpdateTask}},
        index::{IndexView, IndexArea},
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
        list::{SExprView, ListCursorMode, ListEditor, ListEditorStyle},
        sdf::SdfTerm
    },
    crate::{
        process::ProcessLauncher
    },

    nako::{
        stream::{SecondaryStream2d, PrimaryStream2d},
        glam::{Vec2, Vec3, UVec2, IVec2},
        operations::{
            planar::primitives2d::Box2d,
            volumetric::{Color, Union, Round},
        },
    },
    nakorender::{
        backend::{Backend, LayerId, LayerId2d, LayerInfo},
        marp::MarpBackend,
        winit, camera::Camera2d
    },

};

struct TestView {}

impl View for TestView {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for TestView {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        Some(TerminalAtom::from('.'))
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Full
    }
}

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    //let mut term = Terminal::new(term_port.outer());
    //let term_writer = term.get_writer();

    let event_loop = nakorender::winit::event_loop::EventLoop::new();
    let window = nakorender::winit::window::Window::new(&event_loop).unwrap();
    let mut renderer = Arc::new(Mutex::new(nakorender::marp::MarpBackend::new(&window, &event_loop)));
    let mut sdf_term = Arc::new(RwLock::new(SdfTerm::new(renderer.clone())));
    term_port.outer().add_observer(sdf_term.clone());

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

    let mut process_list_editor = ListEditor::new(
        Box::new(|| {
            Arc::new(RwLock::new(
                ProcessLauncher::new()
            ))
        }),
        ListEditorStyle::VerticalSexpr
    );

    table_buf.insert_iter(vec![
        (Point2::new(0, 0), magic.clone()),
        (Point2::new(0, 1), status_chars_port.outer().to_sequence().to_grid_horizontal()),
        (Point2::new(0, 2), magic.clone()),
        (Point2::new(0, 3), process_list_editor.get_term_view())
    ]);

    //compositor.write().unwrap().push(monstera::make_monstera());
    compositor.write().unwrap().push(table_port.outer().flatten());//.offset(Vector2::new(40, 2)));

    process_list_editor.goto(TreeCursor {
        leaf_mode: ListCursorMode::Insert,
        tree_addr: vec![ 0 ]
    });

    event_loop.run(move |event, _target, control_flow|{
        //Set to polling for now, might be overwritten
        //TODO: Maybe we want to use "WAIT" for the ui thread? However, the renderer.lock().unwrap()s don't work that hard
        //if nothing changes. So should be okay for a alpha style programm.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        //now check if a rerender was requested, or if we worked on all
        //events on that batch
        term_port.update();
        renderer.lock().unwrap().set_layer_order(
            vec![
                //vec![ color_layer_id.into() ].into_iter(),
                sdf_term.read().unwrap().get_order().into_iter()
            ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .as_slice()
        );

        match event{
            winit::event::Event::WindowEvent{window_id: _, event: winit::event::WindowEvent::Resized(newsize)} => {
                
            }
            winit::event::Event::WindowEvent{window_id: _, event: winit::event::WindowEvent::KeyboardInput{ device_id, input, is_synthetic }} => {
                if input.state == winit::event::ElementState::Pressed {
                    if let Some(kc) = input.virtual_keycode {
                        match kc {
                            winit::event::VirtualKeyCode::Space => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char(' '))));
                            }
                            winit::event::VirtualKeyCode::Return => {
                                process_list_editor.get_item().unwrap().write().unwrap().launch_pty2()
                            }
                            winit::event::VirtualKeyCode::Key0 |
                            winit::event::VirtualKeyCode::Numpad0 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('0'))));
                            }
                            winit::event::VirtualKeyCode::Key1 |
                            winit::event::VirtualKeyCode::Numpad1 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('1'))));
                            }
                            winit::event::VirtualKeyCode::Key2 |
                            winit::event::VirtualKeyCode::Numpad2 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('2'))));
                            }
                            winit::event::VirtualKeyCode::Key3 |
                            winit::event::VirtualKeyCode::Numpad3 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('3'))));
                            }
                            winit::event::VirtualKeyCode::Key4 |
                            winit::event::VirtualKeyCode::Numpad4 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('4'))));
                            }
                            winit::event::VirtualKeyCode::Key5 |
                            winit::event::VirtualKeyCode::Numpad5 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('5'))));
                            }
                            winit::event::VirtualKeyCode::Key6 |
                            winit::event::VirtualKeyCode::Numpad6 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('6'))));
                            }
                            winit::event::VirtualKeyCode::Key7 |
                            winit::event::VirtualKeyCode::Numpad7 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('7'))));
                            }
                            winit::event::VirtualKeyCode::Key8 |
                            winit::event::VirtualKeyCode::Numpad8 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('8'))));
                            }
                            winit::event::VirtualKeyCode::Key9 |
                            winit::event::VirtualKeyCode::Numpad9 => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('9'))));
                            }
                            winit::event::VirtualKeyCode::A => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('a'))));
                            }
                            winit::event::VirtualKeyCode::B => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('b'))));
                            }
                            winit::event::VirtualKeyCode::C => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('c'))));
                            }
                            winit::event::VirtualKeyCode::D => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('d'))));
                            }
                            winit::event::VirtualKeyCode::E => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('e'))));
                            }
                            winit::event::VirtualKeyCode::F => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('f'))));
                            }
                            winit::event::VirtualKeyCode::G => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('g'))));
                            }
                            winit::event::VirtualKeyCode::H => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('h'))));
                            }
                            winit::event::VirtualKeyCode::I => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('i'))));
                            }
                            winit::event::VirtualKeyCode::J => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('j'))));
                            }
                            winit::event::VirtualKeyCode::K => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('k'))));
                            }
                            winit::event::VirtualKeyCode::L => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('l'))));
                            }
                            winit::event::VirtualKeyCode::M => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('m'))));
                            }
                            winit::event::VirtualKeyCode::N => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('n'))));
                            }
                            winit::event::VirtualKeyCode::O => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('o'))));
                            }
                            winit::event::VirtualKeyCode::P => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('p'))));
                            }
                            winit::event::VirtualKeyCode::Q => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('q'))));
                            }
                            winit::event::VirtualKeyCode::R => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('r'))));
                            }
                            winit::event::VirtualKeyCode::S => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('s'))));
                            }
                            winit::event::VirtualKeyCode::T => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('t'))));
                            }
                            winit::event::VirtualKeyCode::U => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('u'))));
                            }
                            winit::event::VirtualKeyCode::V => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('v'))));
                            }
                            winit::event::VirtualKeyCode::W => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('w'))));
                            }
                            winit::event::VirtualKeyCode::X => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('x'))));
                            }
                            winit::event::VirtualKeyCode::Y => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('y'))));
                            }
                            winit::event::VirtualKeyCode::Z => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('z'))));
                            }
                            winit::event::VirtualKeyCode::Tab => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Insert)));
                            }
                            winit::event::VirtualKeyCode::Delete => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Delete)));
                            }
                            winit::event::VirtualKeyCode::Back => {
                                process_list_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Backspace)));
                            }
                            winit::event::VirtualKeyCode::Left => {
                                process_list_editor.pxev();
                            }
                            winit::event::VirtualKeyCode::Right => {
                                process_list_editor.nexd();
                            }
                            winit::event::VirtualKeyCode::Up => {
                                process_list_editor.up();
                            }
                            winit::event::VirtualKeyCode::Down => {
                                process_list_editor.dn();
                                process_list_editor.goto_home();
                            }
                            winit::event::VirtualKeyCode::Home => {
                                process_list_editor.goto_home();
                            }
                            winit::event::VirtualKeyCode::End => {
                                process_list_editor.goto_end();
                            }
                            _ => {
                            }
                        }
                    }
                }
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                //term_port.update();
                renderer.lock().unwrap().render(&window);
            }
            _ => {},
        }

        status_chars.clear();
        let cur = process_list_editor.get_cursor();

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
        
    });
/*
    async_std::task::spawn(
        async move {

            let tp = term_port.clone();
            async_std::task::spawn(
                async move {
                    loop {
                        tp.update();
                        async_std::task::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
            );
            
            loop {
                let ev = term.next_event().await;
                match ev {
                    TerminalEvent::Resize(new_size) => {
                        cur_size.set(new_size);
                        term_port.inner().get_broadcast().notify(&IndexArea::Full);
                    }
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                    TerminalEvent::Input(Event::Key(Key::Left)) => {
                        process_list_editor.pxev();
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        process_list_editor.nexd();
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => {
                        if process_list_editor.up() == TreeNavResult::Exit {
                            process_list_editor.dn();
                            process_list_editor.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Down)) => {
                        if process_list_editor.dn() == TreeNavResult::Continue {
                            process_list_editor.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        process_list_editor.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        process_list_editor.goto_end();
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        process_list_editor.get_item().unwrap().write().unwrap().launch_pty2();
                    }

                    ev => {
                        if process_list_editor.get_cursor().leaf_mode == ListCursorMode::Select {
                            match ev {
                                TerminalEvent::Input(Event::Key(Key::Char('l'))) => { process_list_editor.up(); },
                                TerminalEvent::Input(Event::Key(Key::Char('a'))) => { process_list_editor.dn(); },
                                TerminalEvent::Input(Event::Key(Key::Char('i'))) => { process_list_editor.pxev(); },
                                TerminalEvent::Input(Event::Key(Key::Char('e'))) => { process_list_editor.nexd(); },
                                TerminalEvent::Input(Event::Key(Key::Char('u'))) => { process_list_editor.goto_home(); },
                                TerminalEvent::Input(Event::Key(Key::Char('o'))) => { process_list_editor.goto_end(); },
                                _ => {
                                    process_list_editor.handle_terminal_event(&ev);
                                }
                            }
                        } else {
                            process_list_editor.handle_terminal_event(&ev);
                        }
                    }
                }

                status_chars.clear();
                let cur = process_list_editor.get_cursor();

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

            drop(term);
            drop(term_port);
        }
    );

    term_writer.show().await.expect("output error!");
*/
}

