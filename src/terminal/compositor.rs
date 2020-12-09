use {
    std::sync::{Arc, RwLock},
    cgmath::Vector2,
    crate::{
        view::{View, Observer},
        port::{ViewPort, InnerViewPort, OuterViewPort},
        terminal::{TerminalAtom}
    }
};

pub struct TerminalCompositor {
    layers: Arc<RwLock<Vec<Arc<dyn View<Key = Vector2<i16>, Value = TerminalAtom>>>>>,
    port: Arc<InnerViewPort<Vector2<i16>, TerminalAtom>>
}

impl TerminalCompositor {
    pub fn new(port: InnerViewPort<Vector2<i16>, TerminalAtom>) -> Self {
        let layers = Arc::new(RwLock::new(Vec::<Arc<dyn View<Key = Vector2<i16>, Value = TerminalAtom>>>::new()));

        port.set_view_fn({
            let layers = layers.clone();
            move |pos| {
                let mut atom = None;

                for l in layers.read().unwrap().iter() {
                    match (atom, l.view(pos)) {
                        (None, next) => atom = next,
                        (Some(last), Some(next)) => atom = Some(next.add_style_back(last.style)),
                        _ => {}
                    }
                }

                atom
            }
        });
        
        TerminalCompositor {
            layers,
            port: Arc::new(port)
        }
    }

    pub fn push(&mut self, v: OuterViewPort<Vector2<i16>, TerminalAtom>) {
        self.layers.write().unwrap().push(v.add_observer(self.port.clone()));
    }
    
    pub fn make_port(&mut self) -> InnerViewPort<Vector2<i16>, TerminalAtom> {
        let port = ViewPort::new();
        self.push(port.outer());
        port.inner()
    }
}

