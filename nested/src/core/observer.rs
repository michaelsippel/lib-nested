use {
    crate::{
        core::{
            View,
            channel::{channel, ChannelSender, ChannelReceiver}
        }
    },
    std::{
        sync::{Arc, Weak}
    },
    std::sync::RwLock
};

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Observer
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub trait Observer<V: View + ?Sized> : Send + Sync {
    fn reset(&mut self, _view: Option<Arc<V>>) {}
    fn notify(&mut self, msg: &V::Msg);
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<V: View + ?Sized, O: Observer<V>> Observer<V> for Arc<RwLock<O>> {
    fn reset(&mut self, view: Option<Arc<V>>) {
        self.write().unwrap().reset(view);
    }

    fn notify(&mut self, msg: &V::Msg) {
        self.write().unwrap().notify(msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait ObserverExt<V: View + ?Sized> : Observer<V> {    
    fn notify_each(&mut self, it: impl IntoIterator<Item = V::Msg>);
}

impl<V: View + ?Sized, T: Observer<V>> ObserverExt<V> for T {
    fn notify_each(&mut self, it: impl IntoIterator<Item = V::Msg>) {
        for msg in it {
            self.notify(&msg);
        }
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Broadcast
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub struct ObserverBroadcast<V: View + ?Sized>
where V::Msg : Send + Sync {
    rx: ChannelReceiver<Vec<V::Msg>>,
    tx: ChannelSender<Vec<V::Msg>>,
    observers: Vec<Weak<RwLock<dyn Observer<V>>>>
}

impl<V: View + ?Sized> ObserverBroadcast<V>
where V::Msg : Clone + Send + Sync {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Vec<V::Msg>>();
        ObserverBroadcast {
            rx, tx,
            observers: Vec::new()
        }
    }

    pub fn add_observer(&mut self, obs: Weak<RwLock<dyn Observer<V>>>) {
        self.cleanup();
        self.observers.push(obs);
    }

    fn cleanup(&mut self) {
        self.observers.retain(|o| o.strong_count() > 0);
    }

    fn iter(&self) -> impl Iterator<Item = Arc<RwLock<dyn Observer<V>>>> + '_ {
        self.observers.iter().filter_map(|o| o.upgrade())
    }

    pub fn update(&self) {
        if let Some(msg_vec) = self.rx.try_recv() {
            for msg in msg_vec {
                for o in self.iter() {
                    o.write().unwrap().notify(&msg);
                }
            }
        }
    }
}

impl<V: View + ?Sized> Observer<V> for ObserverBroadcast<V>
where V::Msg: Clone {
    fn reset(&mut self, view: Option<Arc<V>>) {
        for o in self.iter() {
            o.write().unwrap().reset(view.clone());
        }
    }

    fn notify(&mut self, msg: &V::Msg) {
        self.tx.send(msg.clone());
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    f: F,
    _phantom: std::marker::PhantomData<V>
}

impl<V, F> NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    pub fn new(f: F) -> Self {
        NotifyFnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<V, F> Observer<V> for NotifyFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(&V::Msg) + Send + Sync {
    fn notify(&mut self, msg: &V::Msg) {
        (self.f)(msg);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    f: F,
    _phantom: std::marker::PhantomData<V>
}

impl<V, F> ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    pub fn new(f: F) -> Self {
        ResetFnObserver {
            f,
            _phantom: std::marker::PhantomData
        }
    }
}

impl<V, F> Observer<V> for ResetFnObserver<V, F>
where V: View + ?Sized,
      F: Fn(Option<Arc<V>>) + Send + Sync {
    fn notify(&mut self, _msg: &V::Msg) {}
    fn reset(&mut self, view: Option<Arc<V>>) {
        (self.f)(view);
    }
}


