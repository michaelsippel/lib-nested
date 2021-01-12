pub use {
    std::{
        sync::{Arc, RwLock},
        ops::Range
    },
    crate::{
        core::{
            View,
            Observer,
            ObserverBroadcast,
            ViewPort,
            InnerViewPort,
            OuterViewPort
        },
        index::{IndexView}
    }
};

impl<Key: 'static, Item: 'static> OuterViewPort<dyn IndexView<Key, Item = Item>> {
    pub fn map_item<
        DstItem: Default + 'static,
        F: Fn(&Item) -> DstItem + Send + Sync + 'static
    >(
        &self,
        f: F
    ) -> OuterViewPort<dyn IndexView<Key, Item = DstItem>> {
        let port = ViewPort::new();
        let map = MapIndexItem::new(port.inner(), f);
        self.add_observer(map.clone());
        port.into_outer()
    }
}

pub struct MapIndexItem<Key, DstItem, SrcView, F>
where SrcView: IndexView<Key> + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<Key, Item = DstItem>>>>
}

impl<Key, DstItem, SrcView, F> MapIndexItem<Key, DstItem, SrcView, F>
where Key: 'static,
      DstItem: Default + 'static,
      SrcView: IndexView<Key> + ?Sized + 'static,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync + 'static
{
    fn new(
        port: InnerViewPort<dyn IndexView<Key, Item = DstItem>>,
        f: F
    ) -> Arc<RwLock<Self>> {
        let map = Arc::new(RwLock::new(
            MapIndexItem {
                src_view: None,
                f,
                cast: port.get_broadcast()
            }
        ));

        port.set_view(Some(map.clone()));
        map
    }
}

impl<Key, DstItem, SrcView, F> View for MapIndexItem<Key, DstItem, SrcView, F>
where SrcView: IndexView<Key> + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    type Msg = Key;
}

impl<Key, DstItem, SrcView, F> IndexView<Key> for MapIndexItem<Key, DstItem, SrcView, F>
where DstItem: Default,
      SrcView: IndexView<Key> + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    type Item = DstItem;

    fn get(&self, key: &Key) -> Self::Item {
        if let Some(v) = self.src_view.as_ref() {
            (self.f)(&v.get(key))
        } else {
            DstItem::default()
        }
    }

    fn range(&self) -> Option<Range<Key>> {
        self.src_view.as_ref()?.range()
    }
}

impl<Key, DstItem, SrcView, F> Observer<SrcView> for MapIndexItem<Key, DstItem, SrcView, F>
where SrcView: IndexView<Key> + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        // todo: notify on reset ??
        self.src_view = view;
    }

    fn notify(&self, msg: &Key) {
        self.cast.notify(msg);
    }
}
