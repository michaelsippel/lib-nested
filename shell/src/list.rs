
use {
    std::sync::Arc,
    std::sync::RwLock,
    nested::{
        core::{
            View,
            ViewPort,
            OuterViewPort,
            InnerViewPort,
            Observer,
            ObserverBroadcast
        },
        sequence::{SequenceView, VecBuffer},
        terminal::{Terminal, TerminalAtom, TerminalStyle},
        projection::ProjectionHelper
    }
};

pub struct ListDecorator {
    opening_port: OuterViewPort<dyn SequenceView<Item = char>>,
    closing_port: OuterViewPort<dyn SequenceView<Item = char>>,
    delim_port: OuterViewPort<dyn SequenceView<Item = char>>,
    items: Arc<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>,
    list_style: TerminalStyle,
    item_style: TerminalStyle,

    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>>>,
    proj_helper: ProjectionHelper<Self>
}

impl View for ListDecorator {
    type Msg = usize;
}

impl SequenceView for ListDecorator {
    type Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>;

    fn len(&self) -> Option<usize> {
        Some(self.items.len()? * 2 + 1)
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        let item_idx = idx / 2;
        let list_style = self.list_style.clone();
        let item_style = self.item_style.clone();        
        Some(
            if idx % 2 == 0 {
                if item_idx == 0 {
                    self.opening_port.clone()
                } else if item_idx == self.items.len().unwrap_or(0) {
                    self.closing_port.clone()
                } else {
                    self.delim_port.clone()
                }
                .map(move |c| TerminalAtom::new(*c, list_style))
            } else {
                self.items
                    .get(&item_idx)?
                    .map(move |atom| atom.add_style_back(item_style))
            }
        )
    }
}

impl ListDecorator {
    pub fn new(
        opening_port: OuterViewPort<dyn SequenceView<Item = char>>,
        closing_port: OuterViewPort<dyn SequenceView<Item = char>>,
        delim_port: OuterViewPort<dyn SequenceView<Item = char>>,
        items_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>,
        list_style: TerminalStyle,
        item_style: TerminalStyle,
        out_port: InnerViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        
        let li = Arc::new(RwLock::new(ListDecorator {
            opening_port,
            closing_port,
            delim_port,
            items: proj_helper.new_sequence_arg(
                items_port,
                |s: &mut Self, item_idx| {
                    s.cast.notify(&(item_idx * 2));
                    s.cast.notify(&(item_idx * 2 + 1));
                }
            ),
            list_style,
            item_style,
            cast: out_port.get_broadcast(),
            proj_helper
        }));

        out_port.set_view(Some(li.clone()));
        li
    }

    pub fn lisp_style(
        level: usize,
        items_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>
    ) -> Arc<RwLock<Self>> {
        let opening_port = ViewPort::new();
        let opening = VecBuffer::<char>::with_data("(".chars().collect(), opening_port.inner());

        let closing_port = ViewPort::new();
        let closing = VecBuffer::<char>::with_data(")".chars().collect(), closing_port.inner());

        let delim_port = ViewPort::new();
        let delim = VecBuffer::<char>::with_data(" ".chars().collect(), delim_port.inner());

        Self::new(
            opening_port.outer().to_sequence(),
            closing_port.outer().to_sequence(),
            delim_port.outer().to_sequence(),
            items_port,
            TerminalStyle::fg_color(
                match level {
                    0 => (200, 120, 10),
                    1 => (120, 200, 10),
                    _ => (255, 255, 255)
                }
            ),
            TerminalStyle::fg_color(
                match level {
                    _ => (255, 255, 255)
                }
            ),
            out_port
        )
    }

    pub fn c_style(
        level: usize,
        items_port: OuterViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>,
        out_port: InnerViewPort<dyn SequenceView<Item = OuterViewPort<dyn SequenceView<Item = TerminalAtom>>>>
    ) -> Arc<RwLock<Self>> {
        let opening_port = ViewPort::new();
        let opening = VecBuffer::<char>::with_data("{".chars().collect(), opening_port.inner());

        let closing_port = ViewPort::new();
        let closing = VecBuffer::<char>::with_data("}".chars().collect(), closing_port.inner());

        let delim_port = ViewPort::new();
        let delim = VecBuffer::<char>::with_data(", ".chars().collect(), delim_port.inner());

        Self::new(
            opening_port.outer().to_sequence(),
            closing_port.outer().to_sequence(),
            delim_port.outer().to_sequence(),
            items_port,
            TerminalStyle::fg_color(
                match level {
                    _ => (255, 255, 255)
                }
            ),
            TerminalStyle::fg_color(
                match level {
                    _ => (255, 255, 255)
                }
            ),
            out_port
        )
    }
}

