use {
    std::{
        sync::{Arc, RwLock},
        collections::HashSet
    },
    cgmath::Point2,
    crate::{
        core::{ViewPort, Observer, ObserverExt, ObserverBroadcast, InnerViewPort, OuterViewPort},
        index::{ImplIndexView},
        terminal::{TerminalAtom, TerminalView, TerminalStyle},
        projection::{ProjectionHelper, ProjectionArg}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct LeveledTermView {
    proj_helper: Option<ProjectionHelper<Self>>,

    src: Arc<RwLock<dyn TerminalView>>,
    level: usize,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
}

impl LeveledTermView {
    pub fn new(
        src: OuterViewPort<dyn TerminalView>
    ) -> (Arc<RwLock<Self>>, OuterViewPort<dyn TerminalView>) {
        let port = ViewPort::new();
        let v = Self::with_port(src, port.inner());
        (v, port.into_outer())
    }

    pub fn with_port(
        src_port: OuterViewPort<dyn TerminalView>,
        dst_port: InnerViewPort<dyn TerminalView>
    ) -> Arc<RwLock<Self>> {
        let v = Arc::new(RwLock::new(
            LeveledTermView {
                proj_helper: None,
                src: Arc::new(RwLock::new(Option::<Arc<dyn TerminalView>>::None)),
                level: 0,
                cast: dst_port.get_broadcast()
            }
        ));

        let mut projection_helper = ProjectionHelper::new(Arc::downgrade(&v));

        let (src, src_obs) = projection_helper.new_arg(
            |p: Arc<RwLock<Self>>, pos: &Point2<i16>| {
                p.read().unwrap().cast.notify(pos);
            });

        v.write().unwrap().proj_helper = Some(projection_helper);
        v.write().unwrap().src = src;
        src_port.add_observer(src_obs);
        dst_port.set_view(Some(v.clone()));

        v
    }

    pub fn set_level(&mut self, l: usize) {
        self.level = l;

        // update complete area
        if let Some(a) = self.src.area() {
            self.cast.notify_each(a);
        }
    }    
}

impl ImplIndexView for LeveledTermView {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        self.src.get(pos).map(
            |a| a.add_style_front(
                if self.level > 0 {
                    TerminalStyle::bold(true)
                        .add(TerminalStyle::bg_color((0, 0, 0)))
                } else {
                    TerminalStyle::bold(false)
                })
        )
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        self.src.area()
    }
}

