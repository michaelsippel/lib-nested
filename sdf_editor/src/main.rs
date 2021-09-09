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


struct TermAtomSDF {
    atom: TerminalAtom
}

impl TermAtomSDF {
    fn new(atom: TerminalAtom) -> Self {
        TermAtomSDF {
            atom
        }
    }

    fn update_bg(&self, layer_id: LayerId2d, renderer: &mut MarpBackend) {
        let mut stream = PrimaryStream2d::new();

        let (r,g,b) = self.atom.style.bg_color.unwrap_or((0,0,0));

            stream = stream.push(
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

        renderer.update_sdf_2d(layer_id, stream.build());
    }

    fn update_ch(&self, layer_id: LayerId2d, renderer: &mut MarpBackend) {
        let mut stream = PrimaryStream2d::new();
        
        if let Some(c) = self.atom.c {
            let font = Font::from_path(Path::new("/usr/share/fonts/TTF/FiraCode-Medium.ttf"),0).unwrap();
            let mut ch = Character::from_font(&font, c).with_size(1.0).with_tesselation_factor(0.01);

            let (r,g,b) = self.atom.style.fg_color.unwrap_or((0, 0, 0));

            ch.color = Vec3::new(
                (r as f32 / 255.0).clamp(0.0, 1.0),
                (g as f32 / 255.0).clamp(0.0, 1.0),
                (b as f32 / 255.0).clamp(0.0, 1.0),
            );

            stream = ch.record_character(stream);
        }

        renderer.update_sdf_2d(layer_id, stream.build());
    }
}

struct SdfTerm {
    src_view: Arc<dyn TerminalView>,
    bg_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,
    ch_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,
    renderer: Arc<Mutex<MarpBackend>>
}

impl SdfTerm {
    fn get_order(&mut self) -> Vec<LayerId> {
        vec![
            self.bg_layers.iter().filter(
                |(_pt, (active, _id))| *active
            )
                .collect::<Vec<_>>()
                .into_iter(),
            self.ch_layers.iter().filter(
                |(_pt, (active, _id))| *active
            )
                .collect::<Vec<_>>()
                .into_iter()
        ]
            .into_iter()
            .flatten()
            .map(|(_,(_,id))| (*id).into())
            .collect::<Vec<_>>()
    }

    fn update(&mut self, pt: &Point2<i16>) {
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
        if self.ch_layers.get(pt).is_none() {
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

            self.ch_layers.insert(*pt, (false, id));
        }

        if let Some(atom) = self.src_view.get(pt) {
            let atom_stream_builder = TermAtomSDF::new(atom);
            atom_stream_builder.update_bg(self.bg_layers.get(pt).unwrap().1, &mut *self.renderer.lock().unwrap());
            atom_stream_builder.update_ch(self.ch_layers.get(pt).unwrap().1, &mut *self.renderer.lock().unwrap());

            let has_bg = atom.style.bg_color.is_some();
            let has_fg = atom.c.unwrap_or(' ') != ' ';

            self.bg_layers.get_mut(pt).unwrap().0 = has_bg;
            self.ch_layers.get_mut(pt).unwrap().0 = has_fg;
        } else {
            self.bg_layers.get_mut(pt).unwrap().0 = false;
            self.ch_layers.get_mut(pt).unwrap().0 = false;
        }
    }
}
/*
impl Observer<dyn TerminalView> for SdfTerm {
    fn notify(&mut self, pt: &Point2<i16>) {
        self.update(pt);
        self.update_order();
    }
}
*/
#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    let mut color_editor = ListEditor::new(
        || {
            Arc::new(RwLock::new(PosIntEditor::new(16)))
        },
        nested::list::ListEditorStyle::Path
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

    let color_view = color_port.outer().get_view();

    let cp = color_port.clone();
    let tp = term_port.clone();

    let event_loop = nakorender::winit::event_loop::EventLoop::new();
    let window = nakorender::winit::window::Window::new(&event_loop).unwrap();
    let mut renderer = Arc::new(Mutex::new(nakorender::marp::MarpBackend::new(&window, &event_loop)));

    let mut sdf_term = Arc::new(RwLock::new(SdfTerm {
        src_view: term_port.outer().get_view().unwrap(),
        bg_layers: HashMap::new(),
        ch_layers: HashMap::new(),
        renderer: renderer.clone()
    }));
    //term_port.outer().add_observer(sdf_term.clone());

    async_std::task::spawn(
        async move {
            loop {
                cp.update();
                tp.update();
                
                match term.next_event().await {
                    TerminalEvent::Resize(new_size) => {
                        tp.inner().get_broadcast().notify_each(
                            nested::grid::GridWindowIterator::from(
                                Point2::new(0,0) .. Point2::new(new_size.x, new_size.y)
                            )
                        );

                        
                    }
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                    TerminalEvent::Input(Event::Key(Key::Left)) => {
                        color_editor.pxev();
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        color_editor.nexd();
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => {
                        color_editor.up();
                    }
                    TerminalEvent::Input(Event::Key(Key::Down)) => {
                        color_editor.dn();
                        color_editor.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        color_editor.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        color_editor.goto_end();
                    }
                    event => {
                        color_editor.handle_terminal_event(&event);
                    }
                }
            }
        }
    );

    async_std::task::spawn(async move {
        term_writer.show().await.expect("output error!");
    });


    
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
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                for pt in nested::grid::GridWindowIterator::from(
                    Point2::new(0, 0) .. Point2::new(30, 1)
                ) {
                    sdf_term.write().unwrap().update(&pt);
                }

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
                        sdf_term.write().unwrap().get_order().into_iter(),
                        vec![ color_layer_id.into() ].into_iter()
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

