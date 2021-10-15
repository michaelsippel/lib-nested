
use {
    std::{
        sync::{Arc, RwLock, Mutex},
        collections::HashMap
    },
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    crate::{
        core::{
            View,
            ViewPort,
            Observer,
            ObserverExt,
            OuterViewPort,
            port::UpdateTask
        },
        terminal::{TerminalView}
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
    nako_std::{
        text::Character
    },
    std::{fs::File, io::Read, mem::needs_drop, path::Path},
    font_kit::font::Font,
};

pub struct SdfTerm {
    src_view: Option<Arc<dyn TerminalView>>,
    bg_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,
    fg_layers: HashMap<Point2<i16>, (bool, LayerId2d)>,

    font_height: u32,
    font: Arc<Vec<u8>>,

    renderer: Arc<Mutex<MarpBackend>>
}

impl SdfTerm {
    pub fn new(renderer: Arc<Mutex<MarpBackend>>) -> Self {

        let font_path = Path::new("/usr/share/fonts/TTF/FiraCode-Medium.ttf");
        let mut font_file = File::open(font_path).unwrap();
        let mut font_data = Vec::new();
        font_file.read_to_end(&mut font_data).unwrap();

        SdfTerm {
            src_view: None,
            bg_layers: HashMap::new(),
            fg_layers: HashMap::new(),
            font_height: 30,
            font: Arc::new(font_data),
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
                    extent: UVec2::new(1 + self.font_height / 2, self.font_height),
                    location: IVec2::new(pt.x as i32 * self.font_height as i32 / 2, pt.y as i32 * self.font_height as i32)
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
                    extent: UVec2::new(1 + self.font_height / 2, self.font_height),
                    location: IVec2::new(pt.x as i32 * self.font_height as i32 / 2, pt.y as i32 * self.font_height as i32)
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
                let font_index = 0;
                let fontkit = Font::from_bytes(self.font.clone(), font_index).unwrap();
                let mut ch = Character::from_font(&fontkit, c).with_size(1.0);

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

