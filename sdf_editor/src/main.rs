
use{
    std::{
        sync::{Arc, RwLock, Mutex},
        collections::HashMap
    },
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            Observer,
            ObserverExt,
            OuterViewPort,
            port::UpdateTask
        },
        singleton::{SingletonBuffer, SingletonView},
        sequence::{SequenceView},
        integer::{PosIntEditor},
        terminal::{Terminal, TerminalAtom, TerminalStyle, TerminalView, TerminalCompositor, TerminalEvent, TerminalEditor},
        list::{ListEditor},
        tree_nav::{TreeNav}
    },
    nako::{
        stream::{SecondaryStream2d, PrimaryStream2d},
        glam::{Vec2, Vec3},
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
    nako_std::{
        text::Character
    },
    std::{fs::File, io::Read, mem::needs_drop, path::Path},
    font_kit::font::Font,
};

// projects a Sequence of ints to a color tuple
struct ColorCollector {
    src_view: Option<Arc<dyn SequenceView<Item = u32>>>,
    color: SingletonBuffer<(u8, u8, u8)>
}

impl ColorCollector {
    fn update(&mut self) {
        if let Some(l) = self.src_view.as_ref() {
            let r = l.get(&0).unwrap_or(0);
            let g = l.get(&1).unwrap_or(0);
            let b = l.get(&2).unwrap_or(0);

            self.color.set((r as u8, g as u8, b as u8));
        }
    }
}

impl Observer<dyn SequenceView<Item = u32>> for ColorCollector {
    fn reset(&mut self, new_view: Option<Arc<dyn SequenceView<Item = u32>>>) {
        self.src_view = new_view;
        self.update();
    }

    fn notify(&mut self, idx: &usize) {
        self.update();
    }
}

struct SdfTerm {
    pub src_view: Option<Arc<dyn TerminalView>>,
    bg_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,
    fg_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,
    //font: Arc<RwLock<Font>>,
    renderer: Arc<Mutex<MarpBackend>>
}

impl SdfTerm {
    pub fn new(renderer: Arc<Mutex<MarpBackend>>) -> Self {
        SdfTerm {
            src_view: None,
            bg_layers: HashMap::new(),
            fg_layers: HashMap::new(),
            //font: Arc::new(RwLock::new(Font::from_path(Path::new("/usr/share/fonts/TTF/FiraCode-Medium.ttf"),0).unwrap())),
            renderer
        }
    }

    pub fn get_order(&self) -> Vec<LayerId> {
        vec![
            self.bg_layers.iter(),
            self.fg_layers.iter()
        ]
            .into_iter()
            .flatten()
            .filter_map(
                |(_pt,(active,id))|
                if *active {
                    Some((*id).into())
                } else {
                    None
                })
            .collect::<Vec<_>>()
    }

    pub fn update(&mut self, pt: &Point2<i16>) {
        if self.bg_layers.get(pt).is_none() {
            let id = self.renderer.lock().unwrap().new_layer_2d();

            self.renderer.lock().unwrap().update_camera_2d(
                id.into(),
                Camera2d {
                    extent: Vec2::new(0.5, 1.0),
                    location: Vec2::new(0.0, 0.0),
                    rotation: 0.0
                });
            self.renderer.lock().unwrap().set_layer_info(
                id.into(),
                LayerInfo {
                    extent: (60, 100),
                    location: (pt.x as usize * 60, pt.y as usize * 100)
                });

            self.bg_layers.insert(*pt, (false, id));
        }
        if self.fg_layers.get(pt).is_none() {
            let id = self.renderer.lock().unwrap().new_layer_2d();

            self.renderer.lock().unwrap().update_camera_2d(
                id.into(),
                Camera2d {
                    extent: Vec2::new(0.5, 1.0),
                    location: Vec2::new(0.0, 0.0),
                    rotation: 0.0
                });
            self.renderer.lock().unwrap().set_layer_info(
                id.into(),
                LayerInfo {
                    extent: (60, 100),
                    location: (pt.x as usize * 60, pt.y as usize * 100)
                });

            self.fg_layers.insert(*pt, (false, id));
        }

        if let Some(atom) = self.src_view.get(pt) {

            // background layer
            if let Some((r,g,b)) = atom.style.bg_color {
                let mut stream = PrimaryStream2d::new()
                    .push(
                        SecondaryStream2d::new(
                            Union,
                            Box2d {
                                extent: Vec2::new(0.6, 1.0)
                            }
                        ).push_mod(
                            Color(
                                Vec3::new(
                                    (r as f32 / 255.0).clamp(0.0, 1.0),
                                    (g as f32 / 255.0).clamp(0.0, 1.0),
                                    (b as f32 / 255.0).clamp(0.0, 1.0),
                                )
                            )
                        ).build()
                    );

                self.renderer.lock().unwrap().update_sdf_2d(self.bg_layers.get(pt).unwrap().1, stream.build());
                self.bg_layers.get_mut(pt).unwrap().0 = true;
            } else {
                self.bg_layers.get_mut(pt).unwrap().0 = false;                
            }

            // foreground layer
            if let Some(c) = atom.c {
                let font = Font::from_path(Path::new("/usr/share/fonts/TTF/FiraCode-Light.ttf"),0).unwrap();
                let mut ch = Character::from_font(&font, c).with_size(1.0).with_tesselation_factor(0.01);

                let (r,g,b) = atom.style.fg_color.unwrap_or((0, 0, 0));

                ch.color = Vec3::new(
                    (r as f32 / 255.0).clamp(0.0, 1.0),
                    (g as f32 / 255.0).clamp(0.0, 1.0),
                    (b as f32 / 255.0).clamp(0.0, 1.0),
                );

                let mut stream = PrimaryStream2d::new();
                stream = ch.record_character(stream);

                self.renderer.lock().unwrap().update_sdf_2d(self.fg_layers.get(pt).unwrap().1, stream.build());
                self.fg_layers.get_mut(pt).unwrap().0 = true;
            } else {
                self.fg_layers.get_mut(pt).unwrap().0 = false;                
            }

        } else {
            self.bg_layers.get_mut(pt).unwrap().0 = false;
            self.fg_layers.get_mut(pt).unwrap().0 = false;
        }
    }
}

impl Observer<dyn TerminalView> for SdfTerm {
    fn reset(&mut self, new_view: Option<Arc<dyn TerminalView>>) {
        self.src_view = new_view;

        for pt in self.src_view.area().unwrap_or(vec![]) {
            self.notify(&pt);
        }
    }

    fn notify(&mut self, pt: &Point2<i16>) {
        self.update(pt);
    }
}

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut color_editor = ListEditor::new(
        || {
            Arc::new(RwLock::new(PosIntEditor::new(16)))
        },
        nested::list::ListEditorStyle::HorizontalSexpr
    );

    color_editor.goto(nested::tree_nav::TreeCursor {
        leaf_mode: nested::list::ListCursorMode::Insert,
        tree_addr: vec![ 0 ]
    });

    let color_port = ViewPort::new();
    let color_collector = Arc::new(RwLock::new(ColorCollector {
        src_view: None,
        color: SingletonBuffer::new((200, 200, 0), color_port.inner())
    }));

    let col_seq_port = color_editor.get_data_port().map(
        |sub_editor| sub_editor.read().unwrap().get_value()
    );
    color_port.add_update_hook(Arc::new(col_seq_port.0.clone()));
    col_seq_port.add_observer(
        color_collector.clone()
    );

    compositor.write().unwrap().push(color_editor.get_term_view().offset(Vector2::new(0, 0)));

    let event_loop = nakorender::winit::event_loop::EventLoop::new();
    let window = nakorender::winit::window::Window::new(&event_loop).unwrap();
    let mut renderer = Arc::new(Mutex::new(nakorender::marp::MarpBackend::new(&window, &event_loop)));

    // terminal view
    let mut sdf_term = Arc::new(RwLock::new(SdfTerm::new(renderer.clone())));
    term_port.outer().add_observer(sdf_term.clone());

    // color preview
    let color_view = color_port.outer().get_view();
    let color_layer_id = renderer.lock().unwrap().new_layer_2d();
    renderer.lock().unwrap().update_camera_2d(color_layer_id, Camera2d{
        extent: Vec2::new(4.0, 4.0),
        location: Vec2::new(-2.0, -2.0),
        rotation: 0.0
    });
    renderer.lock().unwrap().set_layer_info(color_layer_id.into(), LayerInfo{
        extent: (600, 600),
        location: (200,100)
    });

    event_loop.run(move |event, _target, control_flow|{
        //Set to polling for now, might be overwritten
        //TODO: Maybe we want to use "WAIT" for the ui thread? However, the renderer.lock().unwrap()s don't work that hard
        //if nothing changes. So should be okay for a alpha style programm.
        *control_flow = winit::event_loop::ControlFlow::Poll;

        //now check if a rerender was requested, or if we worked on all
        //events on that batch
        match event{
            winit::event::Event::WindowEvent{window_id: _, event: winit::event::WindowEvent::Resized(newsize)} => {
                
            }
            winit::event::Event::WindowEvent{window_id: _, event: winit::event::WindowEvent::KeyboardInput{ device_id, input, is_synthetic }} => {
                if input.state == winit::event::ElementState::Pressed {
                    if let Some(kc) = input.virtual_keycode {
                        match kc {
                            winit::event::VirtualKeyCode::Space |
                            winit::event::VirtualKeyCode::Return => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char(' '))));
                            }
                            winit::event::VirtualKeyCode::Key0 |
                            winit::event::VirtualKeyCode::Numpad0 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('0'))));
                            }
                            winit::event::VirtualKeyCode::Key1 |
                            winit::event::VirtualKeyCode::Numpad1 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('1'))));
                            }
                            winit::event::VirtualKeyCode::Key2 |
                            winit::event::VirtualKeyCode::Numpad2 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('2'))));
                            }
                            winit::event::VirtualKeyCode::Key3 |
                            winit::event::VirtualKeyCode::Numpad3 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('3'))));
                            }
                            winit::event::VirtualKeyCode::Key4 |
                            winit::event::VirtualKeyCode::Numpad4 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('4'))));
                            }
                            winit::event::VirtualKeyCode::Key5 |
                            winit::event::VirtualKeyCode::Numpad5 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('5'))));
                            }
                            winit::event::VirtualKeyCode::Key6 |
                            winit::event::VirtualKeyCode::Numpad6 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('6'))));
                            }
                            winit::event::VirtualKeyCode::Key7 |
                            winit::event::VirtualKeyCode::Numpad7 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('7'))));
                            }
                            winit::event::VirtualKeyCode::Key8 |
                            winit::event::VirtualKeyCode::Numpad8 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('8'))));
                            }
                            winit::event::VirtualKeyCode::Key9 |
                            winit::event::VirtualKeyCode::Numpad9 => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('9'))));
                            }
                            winit::event::VirtualKeyCode::A => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('a'))));
                            }
                            winit::event::VirtualKeyCode::B => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('b'))));
                            }
                            winit::event::VirtualKeyCode::C => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('c'))));
                            }
                            winit::event::VirtualKeyCode::D => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('d'))));
                            }
                            winit::event::VirtualKeyCode::E => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('e'))));
                            }
                            winit::event::VirtualKeyCode::F => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Char('f'))));
                            }
                            winit::event::VirtualKeyCode::Tab => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Insert)));
                            }
                            winit::event::VirtualKeyCode::Delete => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Delete)));
                            }
                            winit::event::VirtualKeyCode::Back => {
                                color_editor.handle_terminal_event(&TerminalEvent::Input(Event::Key(Key::Backspace)));
                            }
                            winit::event::VirtualKeyCode::Left => {
                                color_editor.pxev();
                            }
                            winit::event::VirtualKeyCode::Right => {
                                color_editor.nexd();
                            }
                            winit::event::VirtualKeyCode::Up => {
                                color_editor.up();
                            }
                            winit::event::VirtualKeyCode::Down => {
                                color_editor.dn();
                                color_editor.goto_home();
                            }
                            winit::event::VirtualKeyCode::Home => {
                                color_editor.goto_home();
                            }
                            winit::event::VirtualKeyCode::End => {
                                color_editor.goto_end();
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
                color_port.update();
                term_port.update();

                let c = color_view.get();
                let color_stream = PrimaryStream2d::new()
                    .push(
                        SecondaryStream2d::new(
                            Union,
                            Box2d {
                                extent: Vec2::new(0.5, 0.5)
                            }
                        ).push_mod(
                            Color(
                                Vec3::new(
                                    (c.0 as f32 / 255.0).clamp(0.0, 1.0),
                                    (c.1 as f32 / 255.0).clamp(0.0, 1.0),
                                    (c.2 as f32 / 255.0).clamp(0.0, 1.0),
                                )
                            )
                        ).push_mod(
                            Round{radius: 0.2}
                        ).build()
                    ).build();

                renderer.lock().unwrap().update_sdf_2d(color_layer_id, color_stream);
                renderer.lock().unwrap().set_layer_order(
                    vec![
                        vec![ color_layer_id.into() ].into_iter(),
                        sdf_term.read().unwrap().get_order().into_iter()
                    ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .as_slice()
                );

                renderer.lock().unwrap().render(&window);
            }
            _ => {},
        }
        
    })
}

