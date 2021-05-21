use {
    std::{
        sync::{Arc}
    },
    std::sync::RwLock,
    cgmath::Point2,
    crate::{
        core::{InnerViewPort, OuterViewPort, Observer, ObserverBroadcast},
        index::{ImplIndexView},
        terminal::{TerminalAtom, TerminalView},
        projection::ProjectionHelper
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TerminalCompositor {
    layers: Vec<Arc<dyn TerminalView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<Self>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TerminalCompositor {
    pub fn new(
        port: InnerViewPort<dyn TerminalView>
    ) -> Arc<RwLock<Self>> {
        let comp = Arc::new(RwLock::new(
            TerminalCompositor {
                layers: Vec::new(),
                cast: port.get_broadcast(),
                proj_helper: ProjectionHelper::new(port.0.update_hooks.clone())
            }
        ));

        comp.write().unwrap().proj_helper.set_proj(&comp);
        port.set_view(Some(comp.clone()));

        comp
    }

    pub fn push(&mut self, v: OuterViewPort<dyn TerminalView>) {
        self.layers.push(
            self.proj_helper.new_index_arg(
                v,
                |s: &mut Self, pos| {
                    s.cast.notify(pos);
                }
            )
        );
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl ImplIndexView for TerminalCompositor {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

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

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        let mut area = Some(Vec::new());

        for layer in self.layers.iter() {
            if let (
                Some(mut new_area),
                Some(area)
            ) = (
                layer.area(),
                area.as_mut()
            ) {
                area.append(&mut new_area);
            } else {
                area = None;
            }
        }

        area
    }
}

