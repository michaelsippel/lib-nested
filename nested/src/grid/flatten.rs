use {
    std::{
        sync::Arc,
        collections::HashMap
    },
    std::sync::RwLock,
    cgmath::{Point2, Vector2},
    crate::{
        core::{
            View, Observer, ObserverBroadcast, ObserverExt,
            ViewPort, InnerViewPort, OuterViewPort,
            port::UpdateTask
        },
        grid::{GridView, GridWindowIterator},
        index::IndexView,
        projection::ProjectionHelper
    }
};

impl<Item> OuterViewPort<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>
where Item: 'static{
    pub fn flatten(&self) -> OuterViewPort<dyn GridView<Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));
        Flatten::new(self.clone(), port.inner());
        port.into_outer()
    }
}

pub struct Chunk<Item>
where Item: 'static
{
    offset: Vector2<i16>,
    limit: Point2<i16>,
    view: Arc<dyn GridView<Item = Item>>
}

pub struct Flatten<Item>
where Item: 'static
{
    limit: Point2<i16>,
    top: Arc<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>,
    chunks: HashMap<Point2<i16>, Chunk<Item>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn GridView<Item = Item>>>>,
    proj_helper: ProjectionHelper<Self>
}

impl<Item> View for Flatten<Item>
where Item: 'static
{
    type Msg = Point2<i16>;
}

impl<Item> IndexView<Point2<i16>> for Flatten<Item>
where Item: 'static
{
    type Item = Item;

    fn get(&self, idx: &Point2<i16>) -> Option<Self::Item> {
        let chunk_idx = self.get_chunk_idx(*idx)?;        
        let chunk = self.chunks.get(&chunk_idx)?;        
        chunk.view.get(&(*idx - chunk.offset))
    }

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(Point2::new(0, 0) .. self.limit).collect())
    }
}

/* TODO: remove unused projection args (bot-views) if they get replaced by a new viewport  */
impl<Item> Flatten<Item>
where Item: 'static
{
    pub fn new(
        top_port: OuterViewPort<dyn GridView<Item = OuterViewPort<dyn GridView<Item = Item>>>>,
        out_port: InnerViewPort<dyn GridView<Item = Item>>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let flat = Arc::new(RwLock::new(
            Flatten {
                limit: Point2::new(0, 0),
                top: proj_helper.new_index_arg(
                    top_port,
                    |s: &mut Self, chunk_idx| {
                        s.update_chunk(*chunk_idx);
                    }
                ),
                chunks: HashMap::new(),
                cast: out_port.get_broadcast(),
                proj_helper
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
                chunk_port.clone(),
                move |s: &mut Self, idx| {
                    if let Some(chunk) = s.chunks.get(&chunk_idx) {
                        if chunk.limit != chunk.view.range().end {
                            s.update_all_offsets();
                        }
                    }
                    if let Some(chunk) = s.chunks.get(&chunk_idx) {
                        s.cast.notify(&(idx + chunk.offset));
                    }
                }
            );
/*
            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                chunk.view = view;
            } else {
*/
                self.chunks.insert(
                    chunk_idx,
                    Chunk {
                        offset: Vector2::new(-1, -1),
                        limit: Point2::new(-1, -1),
                        view
                    }
                );
                /*
            }
             */

            chunk_port.0.update();
            self.update_all_offsets();
            chunk_port.0.update();
        } else {
            // todo:
            //self.proj_helper.remove_arg();

            let mut dirty_idx = Vec::new();
            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                dirty_idx.extend(
                    GridWindowIterator::from(
                        Point2::new(chunk.offset.x, chunk.offset.y)
                            .. Point2::new(chunk.offset.x + chunk.limit.x, chunk.offset.y + chunk.limit.y))
                );
            }

            self.chunks.remove(&chunk_idx);
            self.cast.notify_each(dirty_idx);
            self.update_all_offsets();
        }
    }

    /// recalculate all chunk offsets
    /// and update size of flattened grid
    fn update_all_offsets(&mut self) {
        let mut dirty_idx = Vec::new();

        let top_range = self.top.range();
        let mut col_widths = vec![0 as i16; (top_range.end.x) as usize];
        let mut row_heights = vec![0 as i16; (top_range.end.y) as usize];

        for chunk_idx in GridWindowIterator::from(self.top.range()) {
            if let Some(chunk) = self.chunks.get_mut(&chunk_idx) {
                let old_offset = chunk.offset;
                let old_limit = chunk.limit;

                chunk.offset = Vector2::new(
                    (0 .. chunk_idx.x as usize).map(|x| col_widths[x]).sum(),
                    (0 .. chunk_idx.y as usize).map(|y| row_heights[y]).sum()
                );
                chunk.limit = chunk.view.range().end;

                col_widths[chunk_idx.x as usize] = std::cmp::max(col_widths[chunk_idx.x as usize], chunk.limit.x);
                row_heights[chunk_idx.y as usize] = std::cmp::max(row_heights[chunk_idx.y as usize], chunk.limit.y);

                if old_offset != chunk.offset {
                    dirty_idx.extend(
                        GridWindowIterator::from(
                            Point2::new(std::cmp::min(old_offset.x, chunk.offset.x),
                                        std::cmp::min(old_offset.y, chunk.offset.y))
                                .. Point2::new(std::cmp::max(old_offset.x + old_limit.x, chunk.offset.x + chunk.limit.x),
                                               std::cmp::max(old_offset.y + old_limit.y, chunk.offset.y + chunk.limit.y)))
                    );
                }
            }
        }

        let old_limit = self.limit;
        self.limit = Point2::new(
            (0 .. top_range.end.x as usize).map(|x| col_widths[x]).sum(),
            (0 .. top_range.end.y as usize).map(|y| row_heights[y]).sum()
        );

        // fixme: dirty hack to mitigate the buggy notifications, not efficien
        dirty_idx.extend(
            GridWindowIterator::from(
                Point2::new(0, 0) .. Point2::new(
                    std::cmp::max(old_limit.x, self.limit.x),
                    std::cmp::max(old_limit.y, self.limit.y)
                )
            )
        );

        self.cast.notify_each(dirty_idx);
    }

    /// given an index in the flattened sequence,
    /// which sub-sequence does it belong to?
    fn get_chunk_idx(&self, glob_pos: Point2<i16>) -> Option<Point2<i16>> {
        for chunk_idx in GridWindowIterator::from(self.top.range()) {
            if let Some(chunk) = self.chunks.get(&chunk_idx) {
                let chunk_range = chunk.view.range();
                let end = chunk_range.end + chunk.offset;

                if glob_pos.x < end.x && glob_pos.y < end.y {
                    return Some(chunk_idx);
                }
            }
        }

        None
    }
}

