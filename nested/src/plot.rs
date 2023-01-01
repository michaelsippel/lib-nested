use {
    cgmath::Point2,
    nested::{
        core::{InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View},
        sequence::{SequenceView},
        index::{IndexArea, IndexView},
        projection::ProjectionHelper,
        terminal::{TerminalAtom, TerminalView},
    },
    std::sync::{Arc, RwLock},
};

pub struct Plot {
    limit: usize,
    data: Arc<dyn SequenceView<Item = usize>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<(), Self>,
}

impl View for Plot {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for Plot {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        if pt.y >= 0 {
            if let Some(cur_val) = self.data.get(&(pt.x as usize)) {
                if cur_val <= self.limit {
                    if pt.y == (self.limit - cur_val) as i16 {
                        return Some(TerminalAtom::from(if cur_val < 4 {
                            'o'
                        } else if cur_val < 8 {
                            'O'
                        } else {
                            '*'
                        }));
                    }
                }
                if pt.x > 0 {
                    if let Some(prev_val) = self.data.get(&((pt.x - 1) as usize)) {
                        if (pt.y > (self.limit - prev_val) as i16
                            && pt.y < (self.limit - cur_val) as i16)
                            || (pt.y < (self.limit - prev_val) as i16
                                && pt.y > (self.limit - cur_val) as i16)
                        {
                            return Some(TerminalAtom::from('.'));
                        }
                    }
                }
            }
        }
        None
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(
            Point2::new(0, 0)..=Point2::new(self.data.len().unwrap_or(0) as i16, self.limit as i16),
        )
    }
}

impl Plot {
    pub fn new(
        data_port: OuterViewPort<dyn SequenceView<Item = usize>>,
        out_port: InnerViewPort<dyn TerminalView>,
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(Plot {
            data: proj_helper.new_sequence_arg((), data_port, |s: &mut Self, idx| {
                let val = s.data.get(idx).unwrap_or(0);

                if val > s.limit {
                    s.limit = val;
                    s.cast.notify(&s.area());
                } else {
                    s.cast.notify(&IndexArea::Range(
                        Point2::new(*idx as i16, 0)..=Point2::new(*idx as i16, s.limit as i16),
                    ));
                }
            }),

            limit: 0,
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.set_view(Some(proj.clone()));

        proj
    }
}
