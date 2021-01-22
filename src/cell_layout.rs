use {
    async_std::stream::StreamExt,
    std::{
        sync::{Arc, RwLock},
        collections::HashMap,
        cmp::{min, max}
    },
    cgmath::{Point2, Vector2},
    crate::{
        core::{InnerViewPort, OuterViewPort, Observer, ObserverExt, ObserverBroadcast, ChannelReceiver, ChannelSender},
        terminal::{TerminalView, TerminalAtom},
        index::{ImplIndexView},
        grid::GridWindowIterator,
        projection::ProjectionArg
    }
};


pub struct Cell {
    view: Arc<dyn TerminalView>,
    _arg: Arc<RwLock<ProjectionArg<dyn TerminalView, CellLayout>>>
}

pub struct CellLayout {
    cells: HashMap<Point2<i16>, Cell>,

    col_widths: Vec<usize>,
    row_heights: Vec<usize>,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,

    send: ChannelSender<Vec<(Point2<i16>, Point2<i16>)>>
}

impl ImplIndexView for CellLayout {
    type Key = Point2<i16>;
    type Value = TerminalAtom;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        let cell_pos = self.get_cell_containing(pos);
        let cell_off = self.get_cell_offset(&cell_pos);

        self.cells.get(&cell_pos)?.view.get(&(pos - cell_off))
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(
            self.cells.iter()
                .flat_map(
                    |(cell_pos, cell)| {
                        let off = self.get_cell_offset(cell_pos);

                        cell.view.area()
                            .unwrap_or(Vec::new())
                            .into_iter()
                            .map(move |p| p + off)
                    }
                ).collect()
        )
    }
}

impl CellLayout {
    pub fn with_port(port: InnerViewPort<dyn TerminalView>) -> Arc<RwLock<Self>> {
        let (send, mut recv) = crate::core::channel::channel();
        let v = Arc::new(RwLock::new(CellLayout {
            cells: HashMap::new(),
            col_widths: Vec::new(),
            row_heights: Vec::new(),
            cast: port.get_broadcast(),
            send
        }));

        /*
         * its is a bit ugly to spawn a task here, but we need the stream to decouple
         * in order to avoid deadlocks
         */
        async_std::task::spawn({
            let l = v.clone();
            async move {
                while let Some((cell_idx, pos)) = recv.next().await {
                    l.write().unwrap().update_cell(&cell_idx, &pos);
                }
            }
        });
        
        port.set_view(Some(v.clone()));
        v
    }

    pub fn set_cell(layout: &Arc<RwLock<Self>>, cell_pos: Point2<i16>, port: OuterViewPort<dyn TerminalView>) {
        let sender = layout.read().unwrap().send.clone();
        let arg = ProjectionArg::new(
            move |s: Arc<RwLock<Self>>, pos: &Point2<i16>| {
                sender.send((cell_pos, *pos));
            }
        );

        layout.write().unwrap().cells.insert(
            cell_pos,
            Cell {
                view: arg.read().unwrap().src.clone(),
                _arg: arg.clone()
            });

        arg.write().unwrap().proj = Arc::downgrade(&layout);
        port.add_observer(arg);
    }

    fn update_col_width(&mut self, col_idx: i16) -> bool {
        let mut max_width = 0;

        for row_idx in 0 .. self.row_heights.len() as i16 {
            if let Some(cell) = self.cells.get(&Point2::new(col_idx, row_idx)) {
                if let Some(area) = cell.view.area() {
                    max_width = max(
                        max_width,
                        area.iter()
                            .map(|pt| pt.x as usize + 1)
                            .max()
                            .unwrap_or(0)
                    );
                }
            }
        }

        let changed = (self.col_widths[col_idx as usize] != max_width);
        self.col_widths[col_idx as usize] = max_width;
        changed
    }

    fn update_row_height(&mut self, row_idx: i16) -> bool {
        let mut max_height = 0;

        for col_idx in 0 .. self.col_widths.len() as i16 {
            if let Some(cell) = self.cells.get(&Point2::new(col_idx, row_idx)) {
                if let Some(area) = cell.view.area() {
                    max_height = max(
                        max_height,
                        area.iter()
                            .map(|pt| pt.y as usize + 1)
                            .max()
                            .unwrap_or(0)
                    );
                }
            }
        }

        let changed = (self.row_heights[row_idx as usize] != max_height);
        self.row_heights[row_idx as usize] = max_height;
        changed
    }

    fn update_cell(&mut self, cell_pos: &Point2<i16>, pos: &Point2<i16>) {
        for _ in self.col_widths.len() as i16 ..= cell_pos.x { self.col_widths.push(0); }
        for _ in self.row_heights.len() as i16 ..= cell_pos.y { self.row_heights.push(0); }

        let cell_off = self.get_cell_offset(cell_pos);
        self.cast.notify(&(pos + cell_off));

        let old_n = self.get_cell_offset(&(cell_pos + Vector2::new(1, 1)));
        let old_width = self.get_width();
        let old_height = self.get_height();

        // does this really have to be recalculated every time ??
        let width_changed = self.update_col_width(cell_pos.x);
        let height_changed = self.update_row_height(cell_pos.y);

        let extent = Point2::new(
            max(self.get_width(), old_width) as i16,
            max(self.get_height(), old_height) as i16
        );
        let new_n = self.get_cell_offset(&(cell_pos + Vector2::new(1, 1)));
        
        /* if a cell updates its size, the complete rectangle to the right is refreshed
         * todo: optimize to use area() of cell views
         */
        if width_changed {
            self.cast.notify_each(GridWindowIterator::from(
                Point2::new(
                    min(old_n.x, new_n.x),
                    0
                )
                    ..
                extent
            ));
        }

        if height_changed {
            self.cast.notify_each(GridWindowIterator::from(
                Point2::new(
                    0,
                    min(old_n.y, new_n.y)
                )
                    ..
                extent
            ));
        }
    }

    fn get_width(&self) -> usize {
        self.col_widths.iter().sum()
    }

    fn get_height(&self) -> usize {
        self.row_heights.iter().sum()
    }

    fn get_cell_offset(&self, cell_pos: &Point2<i16>) -> Vector2<i16> {
        Vector2::new(
            self.col_widths.iter().take(cell_pos.x as usize).sum::<usize>() as i16,
            self.row_heights.iter().take(cell_pos.y as usize).sum::<usize>() as i16
        )
    }

    fn get_cell_containing(&self, glob_pos: &Point2<i16>) -> Point2<i16> {
        Point2::new(
            self.col_widths.iter()
                .fold(
                    (0, 0),
                    |(cell_idx, x), width|
                    (
                        cell_idx + if (x + *width as i16) <= glob_pos.x { 1 } else { 0 },
                        x + *width as i16
                    )).0,

            self.row_heights.iter()
                .fold(
                    (0, 0),
                    |(cell_idx, y), height|
                    (
                        cell_idx + if (y + *height as i16) <= glob_pos.y { 1 } else { 0 },
                        y + *height as i16
                    )).0
        )
    }
}

