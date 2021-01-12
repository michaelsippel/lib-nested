use {
    std::{
        sync::{Arc, RwLock},
        ops::Range
    },
    cgmath::{Point2, Vector2},
    crate::{
        core::{
            View,
            Observer,
            ObserverExt,
            ObserverBroadcast,
            InnerViewPort
        },
        view::{
            index::IndexView
        },
        grid::{GridView, GridWindowIterator}
    }
};

pub struct GridOffset<V: GridView + ?Sized> {
    src: Option<Arc<V>>,
    offset: Vector2<i16>,
    cast: Arc<RwLock<ObserverBroadcast<dyn GridView<Item = V::Item>>>>
}

impl<V: 'static + GridView + ?Sized> GridOffset<V>
where V::Item: Default {
    pub fn new(port: InnerViewPort<dyn GridView<Item = V::Item>>) -> Arc<RwLock<Self>> {
        let offset_view =
            Arc::new(RwLock::new(
                GridOffset::<V> {
                    src: None,
                    offset: Vector2::new(0, 0),
                    cast: port.get_broadcast()
                }
            ));

        port.set_view(Some(offset_view.clone()));
        offset_view
    }

    pub fn set_offset(&mut self, new_offset: Vector2<i16>) {
        let old_range = self.range();
        self.offset = new_offset;
        let new_range = self.range();

        if let Some(old_range) = old_range {
            self.cast.notify_each(GridWindowIterator::from(old_range));
        }
        if let Some(new_range) = new_range {
            self.cast.notify_each(GridWindowIterator::from(new_range));
        }
    }
}

impl<V: GridView + ?Sized> View for GridOffset<V> {
    type Msg = Point2<i16>;
}

impl<V: GridView + ?Sized> IndexView<Point2<i16>> for GridOffset<V>
where V::Item: Default {
    type Item = V::Item;

    fn get(&self, pos: &Point2<i16>) -> Self::Item {
        if let Some(src) = self.src.as_ref() {
            src.get(&(pos - self.offset))
        } else {
            Self::Item::default()
        }
    }

    fn range(&self) -> Option<Range<Point2<i16>>> {
        let src_range = self.src.as_ref()?.range()?;
        Some((src_range.start + self.offset) .. (src_range.end + self.offset))
    }
}

impl<V: GridView + ?Sized> Observer<V> for GridOffset<V>
where V::Item: Default {
    fn reset(&mut self, view: Option<Arc<V>>) {
        let old_range = self.range();
        self.src = view;
        let new_range = self.range();

        if let Some(old_range) = old_range {
            self.cast.notify_each(GridWindowIterator::from(old_range));
        }
        if let Some(new_range) = new_range {
            self.cast.notify_each(GridWindowIterator::from(new_range));
        }
    }

    fn notify(&self, msg: &Point2<i16>) {
        self.cast.notify(&(msg + self.offset));
    }
}

