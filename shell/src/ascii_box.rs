use {
    cgmath::{Point2, Vector2},
    nested::{
        core::{InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View},
        index::{IndexArea, IndexView},
        terminal::{TerminalAtom, TerminalView},
    },
    std::sync::{Arc, RwLock},
};

pub struct AsciiBox {
    content: Option<Arc<dyn TerminalView>>,
    extent: Vector2<i16>,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
}

impl AsciiBox {
    pub fn new(
        extent: Vector2<i16>,
        content_port: OuterViewPort<dyn TerminalView>,
        output_port: InnerViewPort<dyn TerminalView>,
    ) -> Arc<RwLock<Self>> {
        let ascii_box = Arc::new(RwLock::new(AsciiBox {
            content: None,
            extent,
            cast: output_port.get_broadcast(),
        }));

        output_port
            .0
            .update_hooks
            .write()
            .unwrap()
            .push(Arc::new(content_port.0.clone()));
        output_port.set_view(Some(ascii_box.clone()));
        content_port.add_observer(ascii_box.clone());

        ascii_box
    }

    pub fn resize(&mut self, new_extent: Vector2<i16>) {
        if self.extent != new_extent {
            let old_extent = self.extent;
            self.extent = new_extent;
            self.cast.notify(&IndexArea::Range(
                Point2::new(0, 0)
                    ..=Point2::new(
                        1 + std::cmp::max(old_extent.x, new_extent.x),
                        1 + std::cmp::max(old_extent.y, new_extent.y),
                    ),
            ));
        }
    }

    pub fn fit_content(&mut self) {
        if let Some(c) = self.content.as_ref() {
            let p = *c.area().range().end();
            self.resize(Vector2::new(p.x + 1, p.y + 1));
        } else {
            self.resize(Vector2::new(0, 0));
        }
    }
}

impl Observer<dyn TerminalView> for AsciiBox {
    fn reset(&mut self, new_content: Option<Arc<dyn TerminalView>>) {
        self.content = new_content;
        self.fit_content();
    }

    fn notify(&mut self, area: &IndexArea<Point2<i16>>) {
        self.cast.notify(&area.map(|pt| pt + Vector2::new(1, 1)));
        self.fit_content();
    }
}

impl View for AsciiBox {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for AsciiBox {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        if pt.x == 0 || pt.x == self.extent.x + 1 {
            // vertical line
            if pt.y == 0 && pt.x == 0 {
                Some(TerminalAtom::from('╭'))
            } else if pt.y == 0 && pt.x == self.extent.x + 1 {
                Some(TerminalAtom::from('╮'))
            } else if pt.y > 0 && pt.y < self.extent.y + 1 {
                Some(TerminalAtom::from('│'))
            } else if pt.y == self.extent.y + 1 && pt.x == 0 {
                Some(TerminalAtom::from('╰'))
            } else if pt.y == self.extent.y + 1 && pt.x == self.extent.x + 1 {
                Some(TerminalAtom::from('╯'))
            } else {
                None
            }
        } else if pt.y == 0 || pt.y == self.extent.y + 1 {
            // horizontal line
            if pt.x > 0 && pt.x < self.extent.x + 1 {
                Some(TerminalAtom::from('─'))
            } else {
                None
            }
        } else if pt.x > 0 && pt.y > 0 && pt.x < self.extent.x + 1 && pt.y < self.extent.y + 1 {
            Some(
                self.content
                    .get(&(pt - Vector2::new(1, 1)))
                    .unwrap_or(TerminalAtom::from(' ')),
            )
        } else {
            None
        }
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(Point2::new(0, 0)..=Point2::new(1, 1) + self.extent)
    }
}
