use {
    std::sync::Arc,
    std::any::Any,
    std::sync::RwLock,
    crate::core::{
        View,
        Observer,
        ObserverBroadcast,
        NotifyFnObserver,
        ResetFnObserver
    }
};

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 View Port
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub struct ViewPort<V: View + ?Sized> {
    view: Arc<RwLock<Option<Arc<V>>>>,
    cast: Arc<RwLock<ObserverBroadcast<V>>>
}

impl<V: View + ?Sized> ViewPort<V> {
    pub fn new() -> Self {
        ViewPort {
            view: Arc::new(RwLock::new(None)),
            cast: Arc::new(RwLock::new(ObserverBroadcast::new()))
        }
    }

    pub fn with_view(view: Arc<V>) -> Self {
        let port = ViewPort::new();
        port.set_view(Some(view));
        port
    }

    pub fn set_view(&self, view: Option<Arc<V>>) {
        *self.view.write().unwrap() = view.clone();
        self.cast.write().unwrap().reset(view);
    }

    pub fn add_observer(&self, observer: Arc<RwLock<dyn Observer<V>>>) {
        self.cast.write().unwrap().add_observer(Arc::downgrade(&observer));
        observer.write().unwrap().reset(self.view.read().unwrap().clone());
    }

    pub fn inner(&self) -> InnerViewPort<V> {
        InnerViewPort(ViewPort{ view: self.view.clone(), cast: self.cast.clone() })
    }

    pub fn outer(&self) -> OuterViewPort<V> {
        OuterViewPort(ViewPort{ view: self.view.clone(), cast: self.cast.clone() })
    }

    pub fn into_inner(self) -> InnerViewPort<V> {
        InnerViewPort(ViewPort{ view: self.view, cast: self.cast })
    }

    pub fn into_outer(self) -> OuterViewPort<V> {
        OuterViewPort(ViewPort{ view: self.view, cast: self.cast })
    }
}

impl<V: View + ?Sized> Clone for ViewPort<V> {
    fn clone(&self) -> Self {
        ViewPort {
            view: self.view.clone(),
            cast: self.cast.clone()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct InnerViewPort<V: View + ?Sized>(ViewPort<V>);
pub struct OuterViewPort<V: View + ?Sized>(ViewPort<V>);

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: View + ?Sized> InnerViewPort<V> {
    pub fn get_broadcast(&self) -> Arc<RwLock<ObserverBroadcast<V>>> {
        self.0.cast.clone()
    }

    pub fn set_view(&self, view: Option<Arc<V>>) -> Arc<RwLock<ObserverBroadcast<V>>> {
        self.0.set_view(view);
        self.get_broadcast()
    }

    pub fn get_view(&self) -> Option<Arc<V>> {
        self.0.view.read().unwrap().clone()
    }

    pub fn notify(&self, msg: &V::Msg) {
        self.0.cast.read().unwrap().notify(msg);
    }
}

impl<V: View + ?Sized> Clone for InnerViewPort<V> {
    fn clone(&self) -> Self {
        InnerViewPort(self.0.clone())
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: View + ?Sized + 'static> OuterViewPort<V> {
    pub fn get_view(&self) -> Option<Arc<V>> {
        self.0.view.read().unwrap().clone()
    }

    pub fn get_view_arc(&self) -> Arc<RwLock<Option<Arc<V>>>> {
        self.0.view.clone()
    }

    pub fn add_observer(&self, observer: Arc<RwLock<dyn Observer<V>>>) -> Arc<RwLock<Option<Arc<V>>>> {
        self.0.add_observer(observer);
        self.get_view_arc()
    }

    pub fn add_reset_fn<F: Fn(Option<Arc<V>>) + Send + Sync + 'static>(&self, reset: F) -> Arc<RwLock<ResetFnObserver<V, F>>> {
        let obs = Arc::new(RwLock::new(ResetFnObserver::new(reset)));
        self.add_observer(obs.clone());
        obs
    }

    pub fn add_notify_fn<F: Fn(&V::Msg) + Send + Sync + 'static>(&self, notify: F) -> Arc<RwLock<NotifyFnObserver<V, F>>> {
        let obs = Arc::new(RwLock::new(NotifyFnObserver::new(notify)));
        self.add_observer(obs.clone());
        obs
    }
}

impl<V: View + ?Sized> Clone for OuterViewPort<V> {
    fn clone(&self) -> Self {
        OuterViewPort(self.0.clone())
    }
}

/*
impl<V: View + ?Sized + 'static> OuterViewPort<V>
where V::Msg: Clone {
    pub fn into_stream<Data>(
        self,
        reset: impl Fn(Option<Arc<V>>, ChannelSender<Data>) + Send + Sync + 'static
    ) -> ChannelReceiver<Data>
    where Data: ChannelData<Item = V::Msg> + 'static,
          Data::IntoIter: Send + Sync + 'static
    {
        let (s, r) = crate::core::channel::channel::<Data>();
        self.add_observer(Arc::new(s.clone()));
        self.add_reset_fn(
            move |view| { reset(view, s.clone()); }
        );
        r
    }
}
*/

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Debug, Clone)]
pub struct AnyViewPort {
    view: Arc<dyn Any + Send + Sync + 'static>,
    cast: Arc<dyn Any + Send + Sync + 'static>
}

impl AnyViewPort {
    pub fn downcast<V: View + ?Sized + 'static>(self) -> Result<ViewPort<V>, AnyViewPort> {
        match (
            self.view.clone().downcast::<RwLock<Option<Arc<V>>>>(),
            self.cast.clone().downcast::<RwLock<ObserverBroadcast<V>>>()
        ) {
            (Ok(view), Ok(cast)) => Ok(ViewPort{view, cast}),
            _ => Err(self)
        }
    }
}

impl<V: View + ?Sized + 'static> From<ViewPort<V>> for AnyViewPort {
    fn from(port: ViewPort<V>) -> Self {
        AnyViewPort {
            view: port.view as Arc<dyn Any + Send + Sync + 'static>,
            cast: port.cast as Arc<dyn Any + Send + Sync + 'static>
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Debug, Clone)]
pub struct AnyOuterViewPort(AnyViewPort);

#[derive(Debug, Clone)]
pub struct AnyInnerViewPort(AnyViewPort);

impl AnyOuterViewPort {
    pub fn downcast<V: View + ?Sized + 'static>(self) -> Result<OuterViewPort<V>, AnyViewPort> {
        Ok(OuterViewPort(self.0.downcast::<V>()?))
    }
}

impl<V: View + ?Sized + 'static> From<OuterViewPort<V>> for AnyOuterViewPort {
    fn from(port: OuterViewPort<V>) -> Self {
        AnyOuterViewPort(AnyViewPort{
            view: port.0.view as Arc<dyn Any + Send + Sync + 'static>,
            cast: port.0.cast as Arc<dyn Any + Send + Sync + 'static>
        })
    }
}

impl AnyInnerViewPort {
    pub fn downcast<V: View + ?Sized + 'static>(self) -> Result<InnerViewPort<V>, AnyViewPort> {
        Ok(InnerViewPort(self.0.downcast::<V>()?))
    }
}

impl<V: View + ?Sized + 'static> From<InnerViewPort<V>> for AnyInnerViewPort {
    fn from(port: InnerViewPort<V>) -> Self {
        AnyInnerViewPort(AnyViewPort{
            view: port.0.view as Arc<dyn Any + Send + Sync + 'static>,
            cast: port.0.cast as Arc<dyn Any + Send + Sync + 'static>
        })
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

