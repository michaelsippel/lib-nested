use {
    crate::{
        core::{
            port::UpdateTask, InnerViewPort, Observer, ObserverBroadcast, ObserverExt,
            OuterViewPort, View, ViewPort,
        },
        projection::ProjectionHelper,
        singleton::SingletonView,
    },
    std::sync::RwLock,
    std::{collections::BTreeMap, sync::Arc},
};

impl<Item> OuterViewPort<dyn SingletonView<Item = OuterViewPort<dyn SingletonView<Item = Item>>>>
where
    Item: 'static + Default,
{
    pub fn flatten(&self) -> OuterViewPort<dyn SingletonView<Item = Item>> {
        let port = ViewPort::new();
        Flatten::new(self.clone(), port.inner());
        port.into_outer()
    }
}

pub struct Flatten<Item>
where
    Item: 'static + Default,
{
    outer: Arc<dyn SingletonView<Item = OuterViewPort<dyn SingletonView<Item = Item>>>>,
    inner: OuterViewPort<dyn SingletonView<Item = Item>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn SingletonView<Item = Item>>>>,
    proj: ProjectionHelper<usize, Self>
}

impl<Item> View for Flatten<Item>
where
    Item: 'static + Default,
{
    type Msg = ();
}

impl<Item> SingletonView for Flatten<Item>
where
    Item: 'static + Default,
{
    type Item = Item;

    fn get(&self) -> Self::Item {
        if let Some(i) = self.inner.get_view() {
            i.get()
        } else {
            Item::default()
        }
    }
}

impl<Item> Flatten<Item>
where
    Item: 'static + Default,
{
    pub fn new(
        top_port: OuterViewPort<
            dyn SingletonView<Item = OuterViewPort<dyn SingletonView<Item = Item>>>,
        >,
        out_port: InnerViewPort<dyn SingletonView<Item = Item>>,
    ) -> Arc<RwLock<Self>> {
        let mut proj = ProjectionHelper::new(out_port.0.update_hooks.clone());

        let flat = Arc::new(RwLock::new(Flatten {
            outer: proj.new_singleton_arg(0, top_port, |s: &mut Self, _msg| {
                s.inner = s.outer.get();
                s.proj.new_singleton_arg(1, s.inner.clone(), |s: &mut Self, _msg| {
                    s.cast.notify(&());
                });
                //s.inner.0.update();
            }),
            inner: OuterViewPort::default(),
            cast: out_port.get_broadcast(),
            proj,
        }));

        flat.write().unwrap().proj.set_proj(&flat);
        out_port.set_view(Some(flat.clone()));
        flat
    }
}


