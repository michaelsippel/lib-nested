use {
    std::{
        sync::{Arc, Weak, RwLock},
        collections::HashMap,
        boxed::Box,
        cmp::{min, max}
    },
    cgmath::Point2,
    crate::{
        core::{View, ViewPort, InnerViewPort, OuterViewPort, Observer, ObserverExt, ObserverBroadcast},
        index::{ImplIndexView},
        grid::{GridWindowIterator},
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

        {
            let old_view = c.layers[&self.idx].1.clone();
            c.layers.get_mut(&self.idx).unwrap().1 = view.clone();

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

        c.update_range();
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
    area: Option<Vec<Point2<i16>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
}

impl TerminalCompositeView {
    fn update_range(&mut self) {
        self.area = Some(Vec::new());

        for (idx, layer) in self.layers.iter() {
            if let Some(view) = layer.1.as_ref() {
                if let (
                    Some(mut new_area),
                    Some(mut area)
                ) = (
                    view.area(),
                    self.area.as_mut()
                ) {
                    area.append(&mut new_area);
                } else {
                    self.area = None;
                }
            }
        }
    }
}

impl ImplIndexView for TerminalCompositeView {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        let mut atom = None;

        for idx in 0 .. self.idx_count {
            if let Some(l) = self.layers.get(&idx) {
                if let Some(view) = l.1.as_ref() {
                    /*
                    if let Some(range) = view.range() {
                        if pos.x < range.start.x ||
                            pos.x >= range.end.x ||
                            pos.y < range.start.y ||
                            pos.y >= range.end.y {
                                continue;
                            }
                    }
                    */
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
        self.area.clone()
    }
}

pub struct TerminalCompositor {
    view: Arc<RwLock<TerminalCompositeView>>,
    port: InnerViewPort<dyn TerminalView>
}

impl TerminalCompositor {
    pub fn new(
        port: InnerViewPort<dyn TerminalView>
    ) -> Self {
        let view = Arc::new(RwLock::new(
            TerminalCompositeView {
                idx_count: 0,
                layers: HashMap::new(),
                area: Some(Vec::new()),
                cast: port.get_broadcast()
            }
        ));

        port.set_view(Some(view.clone()));
        TerminalCompositor{ view, port }
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

