use {
    std::sync::{Arc, RwLock},
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
    observers: Arc<RwLock<ObserverBroadcast<V>>>
}

impl<V: View + ?Sized> ViewPort<V> {
    pub fn new() -> Self {
        ViewPort {
            view: Arc::new(RwLock::new(None)),
            observers: Arc::new(RwLock::new(ObserverBroadcast::new()))
        }
    }

    pub fn with_view(view: Arc<V>) -> Self {
        let port = ViewPort::new();
        port.set_view(Some(view));
        port
    }

    pub fn set_view(&self, view: Option<Arc<V>>) {
        *self.view.write().unwrap() = view.clone();
        self.observers.write().unwrap().reset(view);
    }

    pub fn add_observer(&self, observer: Arc<RwLock<dyn Observer<V>>>) {
        self.observers.write().unwrap().add_observer(Arc::downgrade(&observer));
        observer.write().unwrap().reset(self.view.read().unwrap().clone());
    }

    pub fn inner(&self) -> InnerViewPort<V> {
        InnerViewPort(ViewPort{ view: self.view.clone(), observers: self.observers.clone() })
    }

    pub fn outer(&self) -> OuterViewPort<V> {
        OuterViewPort(ViewPort{ view: self.view.clone(), observers: self.observers.clone() })
    }

    pub fn into_inner(self) -> InnerViewPort<V> {
        InnerViewPort(ViewPort{ view: self.view, observers: self.observers })
    }

    pub fn into_outer(self) -> OuterViewPort<V> {
        OuterViewPort(ViewPort{ view: self.view, observers: self.observers })
    }
}

impl<V: View + ?Sized> Clone for ViewPort<V> {
    fn clone(&self) -> Self {
        ViewPort {
            view: self.view.clone(),
            observers: self.observers.clone()
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct InnerViewPort<V: View + ?Sized>(ViewPort<V>);
pub struct OuterViewPort<V: View + ?Sized>(ViewPort<V>);

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: View + ?Sized> InnerViewPort<V> {
    pub fn get_broadcast(&self) -> Arc<RwLock<ObserverBroadcast<V>>> {
        self.0.observers.clone()
    }

    pub fn set_view(&self, view: Option<Arc<V>>) -> Arc<RwLock<ObserverBroadcast<V>>> {
        self.0.set_view(view);
        self.get_broadcast()
    }

    pub fn get_view(&self) -> Option<Arc<V>> {
        self.0.view.read().unwrap().clone()
    }

    pub fn notify(&self, msg: &V::Msg) {
        self.0.observers.read().unwrap().notify(msg);
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

