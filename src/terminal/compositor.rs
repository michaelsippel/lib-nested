use {
    std::{
        sync::{Arc, Weak, RwLock},
        collections::HashMap
    },
    cgmath::Point2,
    crate::{
        core::{InnerViewPort, OuterViewPort, Observer, ObserverExt, ObserverBroadcast},
        index::{ImplIndexView},
        terminal::{TerminalAtom, TerminalView}
    }
};

struct CompositeLayer {
    comp: Weak<RwLock<TerminalCompositeView>>,
    idx: usize
}

impl Observer<dyn TerminalView> for CompositeLayer {
    fn reset(&mut self, view: Option<Arc<dyn TerminalView>>) {
        let comp = self.comp.upgrade().unwrap();
        let mut c = comp.write().unwrap();

        let v = &mut c.layers.get_mut(&self.idx).unwrap().1;
        let old_view = v.clone();
        *v = view.clone();
        drop(v);

        if let Some(old_view) = old_view {
            if let Some(area) = old_view.area() {
                c.cast.notify_each(area);
            }
        }

        if let Some(view) = view.as_ref() {
            if let Some(area) = view.area() {
                c.cast.notify_each(area);
            }
        }
    }

    fn notify(&self, pos: &Point2<i16>) {
        self.comp
            .upgrade().unwrap()
            .read().unwrap()
            .cast.notify(pos);
    }
}

pub struct TerminalCompositeView {
    idx_count: usize,
    layers: HashMap<usize, (Arc<RwLock<CompositeLayer>>, Option<Arc<dyn TerminalView>>)>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
}

impl ImplIndexView for TerminalCompositeView {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        let mut atom = None;

        for idx in 0 .. self.idx_count {
            if let Some(l) = self.layers.get(&idx) {
                if let Some(view) = l.1.as_ref() {
                    match (atom, view.get(pos)) {
                        (None, next) => atom = next,
                        (Some(last), Some(next)) => atom = Some(next.add_style_back(last.style)),
                        _ => {}
                    }
                }
            }
        }

        atom
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        let mut area = Some(Vec::new());

        for (_, layer) in self.layers.iter() {
            if let Some(view) = layer.1.as_ref() {
                if let (
                    Some(mut new_area),
                    Some(area)
                ) = (
                    view.area(),
                    area.as_mut()
                ) {
                    area.append(&mut new_area);
                } else {
                    area = None;
                }
            }
        }

        area
    }
}

pub struct TerminalCompositor {
    view: Arc<RwLock<TerminalCompositeView>>
}

impl TerminalCompositor {
    pub fn new(
        port: InnerViewPort<dyn TerminalView>
    ) -> Self {
        let view = Arc::new(RwLock::new(
            TerminalCompositeView {
                idx_count: 0,
                layers: HashMap::new(),
                cast: port.get_broadcast()
            }
        ));

        port.set_view(Some(view.clone()));
        TerminalCompositor{ view }
    }

    pub fn push(&mut self, v: OuterViewPort<dyn TerminalView>) {        
        let mut comp = self.view.write().unwrap();
        let idx = comp.idx_count;
        comp.idx_count += 1;

        let layer = Arc::new(RwLock::new(CompositeLayer {
            comp: Arc::downgrade(&self.view),
            idx: idx
        }));

        comp.layers.insert(idx, (layer.clone(), None));
        drop(comp);

        v.add_observer(layer);
    }
}

