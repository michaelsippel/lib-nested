use {
    std::sync::Arc,
    std::sync::RwLock,
    crate::{
        sequence::{SequenceView},
        core::{
            Observer, ObserverExt, ObserverBroadcast,
            View, ViewPort, OuterViewPort
        }
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn map<
        DstItem: 'static,
        F: Fn(&Item) -> DstItem + Send + Sync + 'static
    >(
        &self,
        f: F
    ) -> OuterViewPort<dyn SequenceView<Item = DstItem>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(MapSequenceItem {
            src_view: None,
            f,
            cast: port.inner().get_broadcast()
        }));

        self.add_observer(map.clone());
        port.inner().set_view(Some(map));
        port.into_outer()
    }

    pub fn filter_map<
        DstItem: Clone + 'static,
        F: Fn(&Item) -> Option<DstItem> + Send + Sync + 'static
    >(
        &self,
        f: F
    ) -> OuterViewPort<dyn SequenceView<Item = DstItem>> {
        self.map(f)
            .filter(|x| x.is_some())
            .map(|x| x.clone().unwrap())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MapSequenceItem<DstItem, SrcView, F>
where SrcView: SequenceView + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    src_view: Option<Arc<SrcView>>,
    f: F,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = DstItem>>>>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> View for MapSequenceItem<DstItem, SrcView, F>
where SrcView: SequenceView + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    type Msg = usize;
}

impl<DstItem, SrcView, F> SequenceView for MapSequenceItem<DstItem, SrcView, F>
where SrcView: SequenceView + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    type Item = DstItem;

    fn len(&self) -> Option<usize> {
        self.src_view.len()
    }

    fn get(&self, idx: &usize) -> Option<DstItem> {
        self.src_view.get(idx).as_ref().map(|item| (self.f)(item))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<DstItem, SrcView, F> Observer<SrcView> for MapSequenceItem<DstItem, SrcView, F>
where SrcView: SequenceView + ?Sized,
      F: Fn(&SrcView::Item) -> DstItem + Send + Sync
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_len = self.len();
        self.src_view = view;
        let new_len = self.len();

        if let Some(len) = old_len { self.cast.notify_each(0 .. len ); }
        if let Some(len) = new_len { self.cast.notify_each(0 .. len ); }
    }

    fn notify(&mut self, msg: &usize) {
        self.cast.notify(msg);
    }
}

