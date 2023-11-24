use {
    r3vi::{
        view::{
            InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View,
            index::*,
        },
        projection::projection_helper::*,
    },
    crate::{TerminalAtom, TerminalView},
    cgmath::Point2,
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TerminalCompositor {
    layers: Vec<Arc<dyn TerminalView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<usize, Self>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TerminalCompositor {
    pub fn new(port: InnerViewPort<dyn TerminalView>) -> Arc<RwLock<Self>> {
        let comp = Arc::new(RwLock::new(TerminalCompositor {
            layers: Vec::new(),
            cast: port.get_broadcast(),
            proj_helper: ProjectionHelper::new(port.0.update_hooks.clone()),
        }));

        comp.write().unwrap().proj_helper.set_proj(&comp);
        port.set_view(Some(comp.clone()));

        comp
    }

    pub fn push(&mut self, v: OuterViewPort<dyn TerminalView>) {
        let idx = self.layers.len();
        self.layers.push(
            self.proj_helper
                .new_index_arg(idx, v, |s: &mut Self, area| {
                    s.cast.notify(area);
                }),
        );
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl View for TerminalCompositor {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for TerminalCompositor {
    type Item = TerminalAtom;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        let mut atom = None;

        for layer in self.layers.iter() {
            match (atom, layer.get(pos)) {
                (None, next) => atom = next,
                (Some(last), Some(next)) => atom = Some(next.add_style_back(last.style)),
                _ => {}
            }
        }

        atom
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        let mut area = IndexArea::Empty;

        for layer in self.layers.iter() {
            area = area.union(layer.area());
        }

        area
    }
}
