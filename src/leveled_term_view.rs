use {
    std::sync::{Arc, RwLock},
    cgmath::Point2,
    crate::{
        core::{ViewPort, Observer, ObserverExt, ObserverBroadcast, InnerViewPort, OuterViewPort},
        index::{ImplIndexView},
        terminal::{TerminalAtom, TerminalView, TerminalStyle},
        projection::ProjectionArg
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct LeveledTermView {
    src: Arc<RwLock<Option<Arc<dyn TerminalView>>>>,
    _src_obs: Arc<RwLock<ProjectionArg<dyn TerminalView, Self>>>,
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
        let src_obs = ProjectionArg::new(
            // we simply forward all messages
            |s: Arc<RwLock<Self>>, msg: &Point2<i16>| {
                s.read().unwrap().cast.notify(msg);
            }
        );

        let v = Arc::new(RwLock::new(
            LeveledTermView {
                src: src_obs.read().unwrap().src.clone(),
                _src_obs: src_obs.clone(),
                level: 0,
                cast: dst_port.get_broadcast()
            }
        ));

        src_obs.write().unwrap().proj = Arc::downgrade(&v);

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

