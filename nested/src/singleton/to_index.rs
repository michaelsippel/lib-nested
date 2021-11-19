use {
    crate::{
        core::{Observer, ObserverBroadcast, OuterViewPort, View, ViewPort},
        grid::GridView,
        index::{IndexArea, IndexView},
        singleton::SingletonView,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SingletonView<Item = Item>> {
    pub fn to_index(&self) -> OuterViewPort<dyn IndexView<(), Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(Singleton2Index {
            src_view: None,
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(map.clone());
        port.inner().set_view(Some(map));
        port.into_outer()
    }

    pub fn to_grid(&self) -> OuterViewPort<dyn GridView<Item = Item>> {
        self.to_index().map_key(
            |_msg: &()| cgmath::Point2::new(0, 0),
            |pt| {
                if pt.x == 0 && pt.y == 0 {
                    Some(())
                } else {
                    None
                }
            },
        )
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Singleton2Index<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    src_view: Option<Arc<SrcView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<(), Item = SrcView::Item>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> View for Singleton2Index<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    type Msg = IndexArea<()>;
}

impl<SrcView> IndexView<()> for Singleton2Index<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    type Item = SrcView::Item;

    fn area(&self) -> IndexArea<()> {
        IndexArea::Full
    }

    fn get(&self, _msg: &()) -> Option<Self::Item> {
        Some(self.src_view.as_ref().unwrap().get())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> Observer<SrcView> for Singleton2Index<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        self.src_view = view;
        self.cast.notify(&IndexArea::Full);
    }

    fn notify(&mut self, _: &()) {
        self.cast.notify(&IndexArea::Full);
    }
}
