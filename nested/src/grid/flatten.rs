use {
    crate::{
        core::{InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View, ViewPort, port::UpdateTask},
        grid::{GridView, GridWindowIterator},
        index::{IndexArea, IndexView},
        projection::ProjectionHelper,
    },
    cgmath::{Point2, Vector2},
    std::sync::RwLock,
    std::{cmp::max, collections::HashMap, sync::Arc},
};

impl<Item> OuterViewPort<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>
where
    Item: 'static,
{
    pub fn flatten(&self) -> OuterViewPort<dyn GridView<Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));
        Flatten::new(self.clone(), port.inner());
        port.into_outer()
    }
}

pub struct Chunk<Item>
where
    Item: 'static,
{
    offset: Vector2<i16>,
    limit: Point2<i16>,
    view: Arc<dyn GridView<Item = Item>>,
}

pub struct Flatten<Item>
where
    Item: 'static,
{
    limit: Point2<i16>,
    top: Arc<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>,
    chunks: HashMap<Point2<i16>, Chunk<Item>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn GridView<Item = Item>>>>,
    proj_helper: ProjectionHelper<Point2<i16>, Self>,
}

impl<Item> View for Flatten<Item>
where
    Item: 'static,
{
    type Msg = IndexArea<Point2<i16>>;
}

impl<Item> IndexView<Point2<i16>> for Flatten<Item>
where
    Item: 'static,
{
    type Item = Item;

    fn get(&self, idx: &Point2<i16>) -> Option<Self::Item> {
        let chunk_idx = self.get_chunk_idx(*idx)?;
        let chunk = self.chunks.get(&chunk_idx)?;
        chunk.view.get(&(*idx - chunk.offset))
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(Point2::new(0, 0)..=self.limit)
    }
}

/* TODO: remove unused projection args (bot-views) if they get replaced by a new viewport  */
impl<Item> Flatten<Item>
where
    Item: 'static,
{
    pub fn new(
        top_port: OuterViewPort<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>,
        out_port: InnerViewPort<dyn GridView<Item = Item>>,
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let flat = Arc::new(RwLock::new(Flatten {
            limit: Point2::new(0, 0),
            top: proj_helper.new_index_arg(
                Point2::new(-1, -1),
                top_port,
                |s: &mut Self, chunk_area| {
                    for chunk_idx in chunk_area.iter() {
                        s.update_chunk(chunk_idx);
                    }
                },
            ),
            chunks: HashMap::new(),
            cast: out_port.get_broadcast(),
            proj_helper,
        }));

        flat.write().unwrap().proj_helper.set_proj(&flat);
        out_port.set_view(Some(flat.clone()));
        flat
    }

    /// the top-sequence has changed the item at chunk_idx,
    /// create a new observer for the contained sub sequence
    fn update_chunk(&mut self, chunk_idx: Point2<i16>) {
        if let Some(chunk_port) = self.top.get(&chunk_idx) {
            let view = self.proj_helper.new_index_arg(
                chunk_idx,
                chunk_port.clone(),
                move |s: &mut Self, area| {
                    if let Some(chunk) = s.chunks.get(&chunk_idx) {
                        if chunk.limit != *chunk.view.area().range().end() {
                            s.update_all_offsets();
                        }
                    }

                    if let Some(chunk) = s.chunks.get(&chunk_idx) {
                        s.cast.notify(&area.map(|pt| pt + chunk.offset));                    
                    }
                },
            );

            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                chunk.view = view;

                let old_limit = chunk.limit;
                let new_limit = *chunk.view.area().range().end();

                self.cast.notify(
                    &IndexArea::Range(
                        Point2::new(chunk.offset.x, chunk.offset.y) ..= Point2::new(chunk.offset.x + max(old_limit.x, new_limit.x), chunk.offset.y + max(old_limit.y, new_limit.y) )));

            } else {
                self.chunks.insert(
                    chunk_idx,
                    Chunk {
                        offset: Vector2::new(-1, -1),
                        limit: Point2::new(-1, -1),
                        view,
                    },
                );
            }
            
            self.update_all_offsets();
        } else {
            self.proj_helper.remove_arg(&chunk_idx);

            if let Some(_chunk) = self.chunks.remove(&chunk_idx) {
                self.update_all_offsets();
            }
        }
    }

    /// recalculate all chunk offsets
    /// and update size of flattened grid
    fn update_all_offsets(&mut self) {
        let top_range = self.top.area().range();
        let mut col_widths = vec![0 as i16; (top_range.end().x + 1) as usize];
        let mut row_heights = vec![0 as i16; (top_range.end().y + 1) as usize];

        for chunk_idx in GridWindowIterator::from(top_range.clone()) {
            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                let chunk_range = chunk.view.area().range();
                let lim = *chunk_range.end();

                col_widths[chunk_idx.x as usize] = max(
                    col_widths[chunk_idx.x as usize],
                    if lim.x < 0 { 0 } else { lim.x + 1 },
                );
                row_heights[chunk_idx.y as usize] = max(
                    row_heights[chunk_idx.y as usize],
                    if lim.y < 0 { 0 } else { lim.y + 1 },
                );
            }
        }

        for chunk_idx in GridWindowIterator::from(top_range.clone()) {
            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                let old_offset = chunk.offset;
                let old_limit = chunk.limit;

                //chunk.limit = Point2::new( col_widths[chunk_idx.x as usize]-1, row_heights[chunk_idx.y as usize]-1 );
                chunk.limit = *chunk.view.area().range().end();

                chunk.offset = Vector2::new(
                    (0..chunk_idx.x as usize).map(|x| col_widths[x]).sum(),
                    (0..chunk_idx.y as usize).map(|y| row_heights[y]).sum(),
                );
/*

                                if old_offset != chunk.offset {
                                    self.cast.notify(
                                        &IndexArea::Range(
                                            Point2::new(
                                                std::cmp::min(old_offset.x, chunk.offset.x),
                                                std::cmp::min(old_offset.y, chunk.offset.y)
                                            )
                                                ..= Point2::new(
                                                    std::cmp::max(old_offset.x + old_limit.x, chunk.offset.x + chunk.limit.x),
                                                    std::cmp::max(old_offset.y + old_limit.y, chunk.offset.y + chunk.limit.y)
                                                )
                                        )
                                    );
                                }
*/
            }
        }

        let old_limit = self.limit;
        self.limit = Point2::new(
            (0..=top_range.end().x)
                .map(|x| col_widths.get(x as usize).unwrap_or(&0))
                .sum::<i16>()
                - 1,
            (0..=top_range.end().y)
                .map(|y| row_heights.get(y as usize).unwrap_or(&0))
                .sum::<i16>()
                - 1,
        );

        self.cast.notify(&IndexArea::Range(
            Point2::new(0, 0)
                ..=Point2::new(
                    max(self.limit.x, old_limit.x),
                    max(self.limit.y, old_limit.y),
                ),
        ));

    }

    /// given an index in the flattened sequence,
    /// which sub-sequence does it belong to?
    fn get_chunk_idx(&self, glob_pos: Point2<i16>) -> Option<Point2<i16>> {
        for chunk_idx in GridWindowIterator::from(self.top.area().range()) {
            if let Some(chunk) = self.chunks.get(&chunk_idx) {
                let end = chunk.limit + chunk.offset;

                if glob_pos.x <= end.x && glob_pos.y <= end.y {
                    return Some(chunk_idx);
                }
            }
        }

        None
    }
}
