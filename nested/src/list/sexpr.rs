use {
    crate::{
        core::{InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View, ViewPort},
        index::IndexArea,
        projection::ProjectionHelper,
        sequence::SequenceView,
        terminal::{make_label, TerminalStyle, TerminalView},
    },
    cgmath::Point2,
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ListDecorator {
    opening_port: OuterViewPort<dyn TerminalView>,
    closing_port: OuterViewPort<dyn TerminalView>,
    delim_port: OuterViewPort<dyn TerminalView>,
    items: Arc<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,

    list_style: TerminalStyle,
    item_style: TerminalStyle,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>>>,
    proj_helper: ProjectionHelper<(), Self>,
}

impl View for ListDecorator {
    type Msg = usize;
}

impl SequenceView for ListDecorator {
    type Item = OuterViewPort<dyn TerminalView>;

    fn len(&self) -> Option<usize> {
        let l = self.items.len()?;
        Some(if l == 0 { 2 } else { 2 * l + 1 })
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let item_idx = idx / 2;
        let list_style = self.list_style.clone();
        let item_style = self.item_style.clone();
        let l = self.items.len().unwrap_or(0);
        Some(if *idx == 0 {
            self.opening_port
                .clone()
                .map_item(move |_, atom| atom.add_style_back(list_style))
        } else if (l == 0 && *idx == 1) || *idx == 2 * l {
            self.closing_port
                .clone()
                .map_item(move |_, atom| atom.add_style_back(list_style))
        } else if idx % 2 == 0 {
            self.delim_port
                .clone()
                .map_item(move |_, atom| atom.add_style_back(list_style))
        } else {
            self.items
                .get(&item_idx)?
                .map_item(move |_, atom| atom.add_style_back(item_style))
        })
    }
}

impl ListDecorator {
    pub fn new(
        opening: &str,
        closing: &str,
        delim: &str,
        level: usize,
        items_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let li = Arc::new(RwLock::new(ListDecorator {
            opening_port: make_label(opening),
            closing_port: make_label(closing),
            delim_port: make_label(delim),
            items: proj_helper.new_sequence_arg((), items_port, |s: &mut Self, item_idx| {
                s.cast.notify(&(*item_idx * 2 + 1));
                s.cast.notify(&(*item_idx * 2 + 2));
            }),
            list_style: TerminalStyle::fg_color(match level {
                0 => (200, 120, 10),
                1 => (120, 200, 10),
                _ => (255, 255, 255),
            }),
            item_style: TerminalStyle::fg_color(match level {
                _ => (255, 255, 255),
            }),
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        li.write().unwrap().proj_helper.set_proj(&li);

        out_port.set_view(Some(li.clone()));
        li
    }
}

pub trait ListDecoration {
    fn decorate(
        &self,
        opening: &str,
        closing: &str,
        delim: &str,
        level: usize,
    ) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>;
}

impl ListDecoration for OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
    fn decorate(
        &self,
        opening: &str,
        closing: &str,
        delim: &str,
        level: usize,
    ) -> OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
        let port = ViewPort::new();
        ListDecorator::new(opening, closing, delim, level, self.clone(), port.inner());
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use crate::{grid::GridView, index::IndexView};

pub struct VerticalSexprDecorator {
    opening_port: OuterViewPort<dyn TerminalView>,
    closing_port: OuterViewPort<dyn TerminalView>,
    items: Arc<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,

    list_style: TerminalStyle,
    item_style: TerminalStyle,

    cast: Arc<RwLock<ObserverBroadcast<dyn GridView<Item = OuterViewPort<dyn TerminalView>>>>>,
    proj_helper: ProjectionHelper<(), Self>,
}

impl View for VerticalSexprDecorator {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for VerticalSexprDecorator {
    type Item = OuterViewPort<dyn TerminalView>;

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(
            Point2::new(0, 0)
                ..=Point2::new(2, std::cmp::max(self.items.len().unwrap() as i16 - 1, 0)),
        )
    }

    fn get(&self, pt: &Point2<i16>) -> Option<Self::Item> {
        if pt.y < 0 {
            return None;
        }
        let item_idx = pt.y as usize;
        let list_style = self.list_style.clone();
        let item_style = self.item_style.clone();
        let l = self.items.len().unwrap_or(0);

        match pt.x {
            0 => {
                if pt.y == 0 {
                    Some(
                        self.opening_port
                            .clone()
                            .map_item(move |_, atom| atom.add_style_back(list_style)),
                    )
                } else {
                    None
                }
            }
            1 => {
                if item_idx < l {
                    Some(
                        self.items
                            .get(&item_idx)?
                            .map_item(move |_, atom| atom.add_style_back(item_style)),
                    )
                } else {
                    None
                }
            }
            2 => {
                if (l == 0 && pt.y == 0) || (item_idx + 1 == l) {
                    Some(
                        self.closing_port
                            .clone()
                            .map_item(move |_, atom| atom.add_style_back(list_style)),
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl VerticalSexprDecorator {
    pub fn new(
        opening: &str,
        closing: &str,
        level: usize,
        items_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>>,
        out_port: InnerViewPort<dyn GridView<Item = OuterViewPort<dyn TerminalView>>>,
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let li = Arc::new(RwLock::new(VerticalSexprDecorator {
            opening_port: make_label(opening),
            closing_port: make_label(closing),
            items: proj_helper.new_sequence_arg((), items_port, |s: &mut Self, item_idx| {
                s.cast.notify(&IndexArea::Range(
                    Point2::new(0, *item_idx as i16)..=Point2::new(2, *item_idx as i16),
                ));
            }),
            list_style: TerminalStyle::fg_color(match level {
                0 => (200, 120, 10),
                1 => (120, 200, 10),
                _ => (255, 255, 255),
            }),
            item_style: TerminalStyle::fg_color(match level {
                _ => (255, 255, 255),
            }),
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        li.write().unwrap().proj_helper.set_proj(&li);

        out_port.set_view(Some(li.clone()));
        li
    }
}

pub trait SExprView {
    fn horizontal_sexpr_view(&self, level: usize) -> OuterViewPort<dyn TerminalView>;
    fn vertical_bar_view(&self, level: usize) -> OuterViewPort<dyn TerminalView>;
    fn vertical_sexpr_view(&self, level: usize) -> OuterViewPort<dyn TerminalView>;
}

impl SExprView for OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn TerminalView>>> {
    fn horizontal_sexpr_view(&self, level: usize) -> OuterViewPort<dyn TerminalView> {
        let port = ViewPort::new();
        ListDecorator::new("(", ")", " ", level, self.clone(), port.inner());

        port.into_outer().to_grid_horizontal().flatten()
    }

    fn vertical_bar_view(&self, level: usize) -> OuterViewPort<dyn TerminalView> {
        let port = ViewPort::new();
        ListDecorator::new("Λ", "V", "|", level, self.clone(), port.inner());

        port.into_outer().to_grid_vertical().flatten()
    }

    fn vertical_sexpr_view(&self, level: usize) -> OuterViewPort<dyn TerminalView> {
        let port = ViewPort::new();
        VerticalSexprDecorator::new("(", ")", level, self.clone(), port.inner());
        port.into_outer().flatten()
    }
}
