use {
    std::{
        sync::{Arc, RwLock}
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
        index::{IndexView},
        grid::{GridView}
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
        let old_area = self.area();
        self.offset = new_offset;
        let new_area = self.area();

        if let Some(area) = old_area { self.cast.notify_each(area); }
        if let Some(area) = new_area { self.cast.notify_each(area); }
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

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(
            self.src.as_ref()?
                .area()?.into_iter()
                .map(|pos| pos + self.offset)
                .collect()
        )
    }
}

impl<V: GridView + ?Sized> Observer<V> for GridOffset<V>
where V::Item: Default {
    fn reset(&mut self, view: Option<Arc<V>>) {
        let old_area = self.area();
        self.src = view;
        let new_area = self.area();

        if let Some(area) = old_area { self.cast.notify_each(area); }
        if let Some(area) = new_area { self.cast.notify_each(area); }
    }

    fn notify(&self, msg: &Point2<i16>) {
        self.cast.notify(&(msg + self.offset));
    }
}

