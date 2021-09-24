
use{
    std::sync::{Arc, RwLock},
    cgmath::{Point2, Vector2},
    nested::{
        core::{
            View,
            ViewPort,
            InnerViewPort,
            OuterViewPort,
            Observer,
            ObserverExt,
            ObserverBroadcast,
            context::{ReprTree, Object, MorphismType, MorphismMode, Context},
            port::{UpdateTask}},
        index::{IndexView},
        grid::{GridWindowIterator},
        terminal::{
            Terminal,
            TerminalStyle,
            TerminalAtom,
            TerminalCompositor,
            TerminalEvent,
            make_label,
            TerminalView,
            TerminalEditor},
    }
};

pub struct AsciiBox {
    content: Option<Arc<dyn TerminalView>>,
    extent: Vector2<i16>,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
}

impl AsciiBox {
    pub fn new(
        extent: Vector2<i16>,
        content_port: OuterViewPort<dyn TerminalView>,
        output_port: InnerViewPort<dyn TerminalView>
    ) -> Arc<RwLock<Self>> {
        let ascii_box = Arc::new(RwLock::new(AsciiBox {
            content: None,
            extent,
            cast: output_port.get_broadcast()
        }));

        output_port.0.update_hooks.write().unwrap().push(Arc::new(content_port.0.clone()));
        output_port.set_view(Some(ascii_box.clone()));
        content_port.add_observer(ascii_box.clone());

        ascii_box
    }

    pub fn resize(&mut self, new_extent: Vector2<i16>) {
        if self.extent != new_extent {
            let old_extent = self.extent;
            self.extent = new_extent;
            self.notify_each(GridWindowIterator::from(Point2::new(0, 0) .. Point2::new(2+std::cmp::max(old_extent.x, new_extent.x), 2+std::cmp::max(old_extent.y, new_extent.y))));
        }
    }

    pub fn fit_content(&mut self) {
        if let Some(c) = self.content.as_ref() {
            let p = c.range().end;
            self.resize(Vector2::new(p.x, p.y));
        } else {
            self.resize(Vector2::new(0, 0));
        }
    }
}

impl Observer<dyn TerminalView> for AsciiBox {
    fn reset(&mut self, new_content: Option<Arc<dyn TerminalView>>) {
        self.content = new_content;
        self.notify_each(GridWindowIterator::from(Point2::new(0, 0) .. Point2::new(self.extent.x+2, self.extent.y+2)));
    }

    fn notify(&mut self, pt: &Point2<i16>) {
        self.cast.notify(&(pt + Vector2::new(1, 1)));
    }
}

impl View for AsciiBox {
    type Msg = Point2<i16>;
}

impl IndexView<Point2<i16>> for AsciiBox {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        if pt.x == 0 || pt.x == self.extent.x+1 {
            // vertical line
            if pt.y == 0 && pt.x == 0 {
                Some(TerminalAtom::from('╭'))
            } else if pt.y == 0 && pt.x == self.extent.x+1 {
                Some(TerminalAtom::from('╮'))
            } else if pt.y > 0 && pt.y < self.extent.y+1 {
                Some(TerminalAtom::from('│'))
            } else if pt.y == self.extent.y+1 && pt.x == 0 {
                Some(TerminalAtom::from('╰'))
            } else if pt.y == self.extent.y+1 && pt.x == self.extent.x+1 {
                Some(TerminalAtom::from('╯'))
            } else {                
                None
            }
        } else if pt.y == 0 || pt.y == self.extent.y+1 {
            // horizontal line
            if pt.x > 0 && pt.x < self.extent.x+1 {
                Some(TerminalAtom::from('─'))
            } else {
                None
            }
        } else if
            pt.x < self.extent.x+1 &&
            pt.y < self.extent.y+1
        {
            self.content.get(&(pt - Vector2::new(1, 1)))
        } else {
            None
        }
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(
            Point2::new(0, 0) .. Point2::new(self.extent.x+2, self.extent.y+2)
        ).collect())
    }

}
