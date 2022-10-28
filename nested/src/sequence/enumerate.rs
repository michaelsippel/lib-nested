use {
    crate::{
        core::{Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort},
        sequence::SequenceView,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn enumerate(&self) -> OuterViewPort<dyn SequenceView<Item = (usize, Item)>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let view = Arc::new(RwLock::new(EnumerateSequence {
            src_view: None,
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(view.clone());
        port.inner().set_view(Some(view));
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct EnumerateSequence<SrcView>
where
    SrcView: SequenceView + ?Sized,
{
    src_view: Option<Arc<SrcView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = (usize, SrcView::Item)>>>>,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> View for EnumerateSequence<SrcView>
where
    SrcView: SequenceView + ?Sized,
{
    type Msg = usize;
}

impl<SrcView> SequenceView for EnumerateSequence<SrcView>
where
    SrcView: SequenceView + ?Sized
{
    type Item = (usize, SrcView::Item);

    fn len(&self) -> Option<usize> {
        self.src_view.len()
    }

    fn get(&self, idx: &usize) -> Option<(usize, SrcView::Item)> {
        self.src_view.get(idx).map(|item| (*idx, item))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView> Observer<SrcView> for EnumerateSequence<SrcView>
where
    SrcView: SequenceView + ?Sized
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_len = self.len();
        self.src_view = view;
        let new_len = self.len();

        if let Some(len) = old_len {
            self.cast.notify_each(0..len);
        }
        if let Some(len) = new_len {
            self.cast.notify_each(0..len);
        }
    }

    fn notify(&mut self, msg: &usize) {
        self.cast.notify(msg);
    }
}


