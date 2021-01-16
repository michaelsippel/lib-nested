use {
    std::{
        sync::{Arc, RwLock}
    },
    crate::{
        core::{
            View, Observer, ObserverExt, ObserverBroadcast,
            ViewPort, InnerViewPort, OuterViewPort
        },
        sequence::SequenceView,
        index::IndexView
    }
};

/// Transforms a SequenceView into IndexView<usize>
pub struct Sequence2Index<SrcView>
where SrcView: SequenceView + ?Sized + 'static {
    src_view: Option<Arc<SrcView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<usize, Item = Option<SrcView::Item>>>>>
}

impl<SrcView> Sequence2Index<SrcView>
where SrcView: SequenceView + ?Sized + 'static {
    pub fn new(
        port: InnerViewPort<dyn IndexView<usize, Item = Option<SrcView::Item>>>
    ) -> Arc<RwLock<Self>> {
        let s2i = Arc::new(RwLock::new(
            Sequence2Index {
                src_view: None,
                cast: port.get_broadcast()
            }
        ));
        port.set_view(Some(s2i.clone()));
        s2i
    }
}

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn to_index(&self) -> OuterViewPort<dyn IndexView<usize, Item = Option<Item>>> {
        let port = ViewPort::new();
        self.add_observer(Sequence2Index::new(port.inner()));
        port.into_outer()
    }
}

impl<SrcView> View for Sequence2Index<SrcView>
where SrcView: SequenceView + ?Sized + 'static {
    type Msg = usize;
}

impl<SrcView> IndexView<usize> for Sequence2Index<SrcView>
where SrcView: SequenceView + ?Sized + 'static {
    type Item = Option<SrcView::Item>;

    fn get(&self, key: &usize) -> Self::Item {
        if let Some(v) = self.src_view.as_ref() {
            if *key < v.len().unwrap_or(usize::MAX) {
                return Some(v.get(*key));
            }
        }
        None
    }

    fn area(&self) -> Option<Vec<usize>> {
        if let Some(v) = self.src_view.as_ref() {
            if let Some(len) = v.len() {
                return Some((0 .. len).collect());
            }
        }
        None
    }
}

impl<SrcView> Observer<SrcView> for Sequence2Index<SrcView>
where SrcView: SequenceView + ?Sized + 'static {
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_area = self.area();
        self.src_view = view;
        let new_area = self.area();

        if let Some(area) = old_area { self.cast.notify_each(area); }
        if let Some(area) = new_area { self.cast.notify_each(area); }
    }

    fn notify(&self, msg: &usize) {
        self.cast.notify(msg);
    }
}

