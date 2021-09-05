use{
    std::sync::{Arc, RwLock},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            Observer,
            OuterViewPort,
            port::UpdateTask
        },
        singleton::{SingletonBuffer, SingletonView},
        sequence::{SequenceView},
        integer::{PosIntEditor},
        terminal::{Terminal, TerminalCompositor, TerminalEvent, TerminalEditor},
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
    nakorender::{backend::{Backend, LayerInfo}, winit, camera::Camera2d}
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
            println!("Set r: {}", r);
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
        nested::list::ListEditorStyle::Clist
    );

    color_editor.goto(nested::tree_nav::TreeCursor {
        leaf_mode: nested::list::ListCursorMode::Insert,
        tree_addr: vec![ 0 ]
    });

    let color_port = ViewPort::new();
    let color_collector = Arc::new(RwLock::new(ColorCollector {
        src_view: None,
        color: SingletonBuffer::new((0, 0, 0), color_port.inner())
    }));

    color_editor.get_data_port().map(
        |sub_editor| sub_editor.read().unwrap().get_value()
    ).add_observer(
        color_collector
    );

    compositor.write().unwrap().push(color_editor.get_term_view().offset(Vector2::new(2, 2)));

    let color_view = color_port.outer().get_view();
    
    async_std::task::spawn(
        async move {
            loop {
                term_port.update();
                color_port.update();
                match term.next_event().await {
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


    let event_loop = nakorender::winit::event_loop::EventLoop::new();
    let window = nakorender::winit::window::Window::new(&event_loop).unwrap();
    let mut renderer = nakorender::marp::MarpBackend::new(&window, &event_loop);

    let id = renderer.new_layer_2d();
    renderer.set_layer_order(&[id.into()]);
    renderer.update_camera_2d(id, Camera2d{
        extent: Vec2::new(2.0, 2.0),
        location: Vec2::new(-1.0, -1.0),
        rotation: 0.0
    });
    
    event_loop.run(move |event, _target, control_flow|{
        //Set to polling for now, might be overwritten
        //TODO: Maybe we want to use "WAIT" for the ui thread? However, the renderers don't work that hard
        //if nothing changes. So should be okay for a alpha style programm.
        *control_flow = winit::event_loop::ControlFlow::Poll;
        

        
        //now check if a rerender was requested, or if we worked on all
        //events on that batch
        match event{
            winit::event::Event::WindowEvent{window_id: _, event: winit::event::WindowEvent::Resized(newsize)} => {
                //update layer to cover whole window again.
                renderer.set_layer_info(id.into(), LayerInfo{
                    extent: (newsize.width as usize, newsize.height as usize),
                    location: (0,0)
                });
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                let new_sdf = sdf_from_color(color_view.get());
                renderer.update_sdf_2d(id, new_sdf);

                renderer.render(&window);
            }
            _ => {},
        }
        
    })
}



fn sdf_from_color(c: (u8, u8, u8)) -> PrimaryStream2d{
    PrimaryStream2d::new()
        .push(
            SecondaryStream2d::new(
                Union,
                Box2d{extent: Vec2::new(0.5, 0.5)}
            ).push_mod(Color(
                Vec3::new(
                    (c.0 as f32 / 255.0).clamp(0.0, 1.0),
                    (c.1 as f32 / 255.0).clamp(0.0, 1.0),
                    (c.2 as f32 / 255.0).clamp(0.0, 1.0),
                )
            )).push_mod(Round{radius: 0.2}).build()
            ).build()
}
