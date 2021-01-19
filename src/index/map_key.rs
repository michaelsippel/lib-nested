pub use {
    std::{
        sync::{Arc, RwLock},
        boxed::Box
    },
    crate::{
        core::{
            View,
            Observer,
            ObserverExt,
            ObserverBroadcast,
            ViewPort,
            InnerViewPort,
            OuterViewPort
        },
        index::{IndexView}
    }
};

impl<SrcKey: 'static, Item: 'static> OuterViewPort<dyn IndexView<SrcKey, Item = Item>> {
    pub fn map_key<
        DstKey: 'static,
        F1: Fn(&SrcKey) -> DstKey + Send + Sync + 'static,
        F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync + 'static,
    >(
        &self,
        f1: F1,
        f2: F2
    ) -> OuterViewPort<dyn IndexView<DstKey, Item = Item>> {
        let port = ViewPort::new();
        let map = MapIndexKey::new(port.inner(), f1, f2);
        self.add_observer(map.clone());
        port.into_outer()
    }
}

pub struct MapIndexKey<DstKey, SrcKey, SrcView, F1, F2>
where SrcView: IndexView<SrcKey> + ?Sized,
      F1: Fn(&SrcKey) -> DstKey + Send + Sync,
      F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync,
{
    src_view: Option<Arc<SrcView>>,
    f1: F1,
    f2: F2,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<DstKey, Item = SrcView::Item>>>>
}

impl<DstKey, SrcKey, SrcView, F1, F2> MapIndexKey<DstKey, SrcKey, SrcView, F1, F2>
where DstKey: 'static,
      SrcKey: 'static,
      SrcView: IndexView<SrcKey> + ?Sized + 'static,
      SrcView::Item: 'static,
      F1: Fn(&SrcKey) -> DstKey + Send + Sync + 'static,
      F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync + 'static,
{
    fn new(
        port: InnerViewPort<dyn IndexView<DstKey, Item = SrcView::Item>>,
        f1: F1,
        f2: F2
    ) -> Arc<RwLock<Self>> {
        let map = Arc::new(RwLock::new(
            MapIndexKey {
                src_view: None,
                f1,
                f2,
                cast: port.get_broadcast()
            }
        ));

        port.set_view(Some(map.clone()));
        map
    }
}

impl<DstKey, SrcKey, SrcView, F1, F2> View for MapIndexKey<DstKey, SrcKey, SrcView, F1, F2>
where SrcView: IndexView<SrcKey> + ?Sized,
      F1: Fn(&SrcKey) -> DstKey + Send + Sync,
      F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync,
{
    type Msg = DstKey;
}

impl<DstKey, SrcKey, SrcView, F1, F2> IndexView<DstKey> for MapIndexKey<DstKey, SrcKey, SrcView, F1, F2>
where SrcView: IndexView<SrcKey> + ?Sized,
      F1: Fn(&SrcKey) -> DstKey + Send + Sync,
      F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync,
{
    type Item = SrcView::Item;

    fn get(&self, key: &DstKey) -> Option<Self::Item> {
        self.src_view.get(&(self.f2)(key)?)
    }

    fn area(&self) -> Option<Vec<DstKey>> {
        Some(self.src_view.area()?.iter().map(&self.f1).collect())
    }
}

impl<DstKey, SrcKey, SrcView, F1, F2> Observer<SrcView> for MapIndexKey<DstKey, SrcKey, SrcView, F1, F2>
where SrcView: IndexView<SrcKey> + ?Sized,
      F1: Fn(&SrcKey) -> DstKey + Send + Sync,
      F2: Fn(&DstKey) -> Option<SrcKey> + Send + Sync,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_area = self.area();
        self.src_view = view;
        let new_area = self.area();

        if let Some(area) = old_area { self.cast.notify_each(area); }
        if let Some(area) = new_area { self.cast.notify_each(area); }
    }

    fn notify(&self, msg: &SrcKey) {
        self.cast.notify(&(self.f1)(msg));
    }
}

