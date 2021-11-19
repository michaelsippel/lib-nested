use {
    crate::{
        core::{InnerViewPort, Observer, ObserverBroadcast, OuterViewPort, View, ViewPort},
        grid::GridView,
        index::{IndexArea, IndexView},
        sequence::SequenceView,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

/// Transforms a SequenceView into IndexView<usize>
pub struct Sequence2Index<SrcView>
where
    SrcView: SequenceView + ?Sized + 'static,
{
    src_view: Option<Arc<SrcView>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<usize, Item = SrcView::Item>>>>,
}

impl<SrcView> Sequence2Index<SrcView>
where
    SrcView: SequenceView + ?Sized + 'static,
{
    pub fn new(
        port: InnerViewPort<dyn IndexView<usize, Item = SrcView::Item>>,
    ) -> Arc<RwLock<Self>> {
        let s2i = Arc::new(RwLock::new(Sequence2Index {
            src_view: None,
            cast: port.get_broadcast(),
        }));
        port.set_view(Some(s2i.clone()));
        s2i
    }
}

impl<Item: 'static> OuterViewPort<dyn SequenceView<Item = Item>> {
    pub fn to_index(&self) -> OuterViewPort<dyn IndexView<usize, Item = Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));
        self.add_observer(Sequence2Index::new(port.inner()));
        port.into_outer()
    }

    pub fn to_grid_horizontal(&self) -> OuterViewPort<dyn GridView<Item = Item>> {
        self.to_index().to_grid_horizontal()
    }

    pub fn to_grid_vertical(&self) -> OuterViewPort<dyn GridView<Item = Item>> {
        self.to_index().to_grid_vertical()
    }
}

impl<SrcView> View for Sequence2Index<SrcView>
where
    SrcView: SequenceView + ?Sized + 'static,
{
    type Msg = IndexArea<usize>;
}

impl<SrcView> IndexView<usize> for Sequence2Index<SrcView>
where
    SrcView: SequenceView + ?Sized + 'static,
{
    type Item = SrcView::Item;

    fn get(&self, key: &usize) -> Option<Self::Item> {
        self.src_view.get(key)
    }

    fn area(&self) -> IndexArea<usize> {
        if let Some(len) = self.src_view.len() {
            if len > 0 {
                IndexArea::Range(0..=len - 1)
            } else {
                IndexArea::Empty
            }
        } else {
            IndexArea::Full
        }
    }
}

impl<SrcView> Observer<SrcView> for Sequence2Index<SrcView>
where
    SrcView: SequenceView + ?Sized + 'static,
{
    fn reset(&mut self, view: Option<Arc<SrcView>>) {
        let old_area = self.area();
        self.src_view = view;

        self.cast.notify(&old_area);
        self.cast.notify(&self.area());
    }

    fn notify(&mut self, idx: &usize) {
        self.cast.notify(&IndexArea::Set(vec![*idx]));
    }
}
