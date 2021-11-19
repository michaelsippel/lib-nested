use {
    crate::{
        core::{Observer, ObserverBroadcast, ObserverExt, OuterViewPort, View, ViewPort},
        sequence::SequenceView,
    },
    std::sync::Arc,
    std::sync::RwLock,
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: SequenceView + ?Sized + 'static> OuterViewPort<V> {
    pub fn filter<P: Fn(&V::Item) -> bool + Send + Sync + 'static>(
        &self,
        pred: P,
    ) -> OuterViewPort<dyn SequenceView<Item = V::Item>> {
        let port = ViewPort::new();
        port.add_update_hook(Arc::new(self.0.clone()));

        let filter = Arc::new(RwLock::new(Filter {
            src_view: None,
            pred,
            old_preds: RwLock::new(Vec::new()),
            cast: port.inner().get_broadcast(),
        }));

        self.add_observer(filter.clone());
        port.inner().set_view(Some(filter));
        port.into_outer()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

struct Filter<SrcView, P>
where
    SrcView: SequenceView + ?Sized + 'static,
    P: Fn(&SrcView::Item) -> bool + Send + Sync + 'static,
{
    src_view: Option<Arc<SrcView>>,
    pred: P,
    old_preds: RwLock<Vec<bool>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SequenceView<Item = SrcView::Item>>>>,
}

impl<SrcView, P> Filter<SrcView, P>
where
    SrcView: SequenceView + ?Sized + 'static,
    P: Fn(&SrcView::Item) -> bool + Send + Sync + 'static,
{
    fn get_offset(&self, idx: usize) -> usize {
        if let Some(v) = self.src_view.clone() {
            let mut i = 0;
            let mut j = 0;
            let mut offset = 0;

            while let (Some(x), true) = (v.get(&i), j <= idx) {
                if (self.pred)(&x) {
                    j += 1;
                } else {
                    offset += 1;
                }
                i += 1;
            }

            offset
        } else {
            0
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView, P> View for Filter<SrcView, P>
where
    SrcView: SequenceView + ?Sized + 'static,
    P: Fn(&SrcView::Item) -> bool + Send + Sync + 'static,
{
    type Msg = usize;
}

impl<SrcView, P> SequenceView for Filter<SrcView, P>
where
    SrcView: SequenceView + ?Sized + 'static,
    P: Fn(&SrcView::Item) -> bool + Send + Sync + 'static,
{
    type Item = SrcView::Item;

    fn len(&self) -> Option<usize> {
        if let Some(src_len) = self.src_view.len() {
            Some(src_len - self.get_offset(src_len))
        } else {
            None
        }
    }

    fn get(&self, idx: &usize) -> Option<Self::Item> {
        self.src_view.get(&(idx + self.get_offset(*idx)))
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<SrcView, P> Observer<SrcView> for Filter<SrcView, P>
where
    SrcView: SequenceView + ?Sized + 'static,
    P: Fn(&SrcView::Item) -> bool + Send + Sync + 'static,
{
    fn reset(&mut self, new_src: Option<Arc<SrcView>>) {
        let old_len = self.len();
        self.src_view = new_src;
        self.old_preds = RwLock::new(Vec::new());
        let new_len = self.len();

        if let Some(len) = old_len {
            self.cast.notify_each(0..len);
        }
        if let Some(len) = new_len {
            self.cast.notify_each(0..len);
        }
    }

    fn notify(&mut self, idx: &usize) {
        let l = self.len().unwrap_or(0) + 1;
        let np = if let Some(x) = self.src_view.get(idx) {
            (self.pred)(&x)
        } else {
            false
        };

        let mut opds = self.old_preds.write().unwrap();

        opds.resize_with(1 + *idx, || false);
        let op = opds.get(*idx).cloned().unwrap_or(false);
        *opds.get_mut(*idx).unwrap() = np;

        drop(opds);

        let i = (0..*idx)
            .map(|j| {
                if let Some(x) = self.src_view.get(&j) {
                    if (self.pred)(&x) {
                        1
                    } else {
                        0
                    }
                } else {
                    0
                }
            })
            .sum();

        if np != op {
            self.cast.notify_each(i..l);
        } else {
            self.cast.notify(&i);
        }
    }
}
