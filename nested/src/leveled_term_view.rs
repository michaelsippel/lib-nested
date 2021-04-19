use {
    std::{
        sync::{Arc},
        collections::HashSet
    },
    std::sync::RwLock,
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
    src: Arc<RwLock<dyn TerminalView>>,
    level: usize,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<Self>
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
        let mut proj_helper = ProjectionHelper::new();

        let v = Arc::new(RwLock::new(
            LeveledTermView {
                src: proj_helper.new_index_arg(
                    src_port,
                    |p: &mut Self, pos: &Point2<i16>| {
                        p.cast.notify(pos);
                    }),
                level: 0,
                cast: dst_port.get_broadcast(),
                proj_helper
            }
        ));

        v.write().unwrap().proj_helper.set_proj(&v);
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

