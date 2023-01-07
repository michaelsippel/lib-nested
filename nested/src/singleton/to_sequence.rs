use {
    crate::{
        core::{Observer, ObserverBroadcast, OuterViewPort, View, ViewPort},
        sequence::{SequenceView},
        singleton::SingletonView,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SingletonView<Item = Item>> {
    pub fn to_sequence(&self) -> OuterViewPort<dyn SequenceView<Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let map = Arc::new(RwLock::new(Singleton2Sequence {
            src_view: None,
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(map.clone());
        port.inner().set_view(Some(map));
        port.into_outer()
    }    
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Singleton2Sequence<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    src_view: Option<Arc<SrcView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = SrcView::Item>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> View for Singleton2Sequence<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    type Msg = usize;
}

impl<SrcView> SequenceView for Singleton2Sequence<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    type Item = SrcView::Item;

    fn get(&self, _idx: &usize) -> Option<Self::Item> {
        Some(self.src_view.as_ref()?.get())
    }

    fn len(&self) -> Option<usize> {
        Some(if self.src_view.is_some() { 1 } else { 0 })
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> Observer<SrcView> for Singleton2Sequence<SrcView>
where
    SrcView: SingletonView + ?Sized,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        self.src_view = view;
        self.cast.notify(&0);
    }

    fn notify(&mut self, _: &()) {
        self.cast.notify(&0);
    }
}
