use {
    core::{
        task::{Poll, Context, Waker},
        pin::Pin
    },
    std::{
        sync::{Arc, Mutex},
        collections::HashSet,
        hash::Hash
    },
    async_std::{
        stream::Stream
    },

    crate::{
        core::{Observer}
    }
};


                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                  Traits
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub trait ChannelData : Default + IntoIterator + Send + Sync {
    fn channel_insert(&mut self, x: Self::Item);
}


                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
               Queue Channel
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
impl<T> ChannelData for Vec<T>
where T: Send + Sync {
    fn channel_insert(&mut self, x: T) {
        self.push(x);
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                 Set Channel
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
impl<T> ChannelData for HashSet<T>
where T: Eq + Hash + Send + Sync {
    fn channel_insert(&mut self, x: T) {
        self.insert(x);
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
             Singleton Channel
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
impl<T> ChannelData for Option<T>
where T: Send + Sync {
    fn channel_insert(&mut self, x: T) {
        *self = Some(x);
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                  Channel
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
struct ChannelState<Data: ChannelData> {
    send_buf: Option<Data>,
    recv_iter: Option<Data::IntoIter>,
    num_senders: usize,
    waker: Option<Waker>
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct ChannelSender<Data: ChannelData>(Arc<Mutex<ChannelState<Data>>>);
pub struct ChannelReceiver<Data: ChannelData>(Arc<Mutex<ChannelState<Data>>>);

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Data: ChannelData> ChannelSender<Data>
where Data::IntoIter: Send + Sync {
    pub fn send(&self, msg: Data::Item) {
        let mut state =  self.0.lock().unwrap();

        if state.send_buf.is_none() {
            state.send_buf = Some(Data::default());
        }

        state.send_buf.as_mut().unwrap().channel_insert(msg);

        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

use crate::core::View;
impl<V: View + ?Sized, Data: ChannelData<Item = V::Msg>> Observer<V> for ChannelSender<Data>
where V::Msg: Clone, Data::IntoIter: Send + Sync {
    fn notify(&self, msg: &V::Msg) {
        self.send(msg.clone());
    }
}

impl<Data: ChannelData> Clone for ChannelSender<Data> {
    fn clone(&self) -> Self {
        self.0.lock().unwrap().num_senders += 1;
        ChannelSender(self.0.clone())
    }
}

impl<Data: ChannelData> Drop for ChannelSender<Data> {
    fn drop(&mut self) {
        let mut state = self.0.lock().unwrap();
        state.num_senders -= 1;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Data: ChannelData> ChannelReceiver<Data> {
    pub async fn recv(&self) -> Option<Data> {
        ChannelRead(self.0.clone()).await
    }
}

struct ChannelRead<Data: ChannelData>(Arc<Mutex<ChannelState<Data>>>);
impl<Data: ChannelData> std::future::Future for ChannelRead<Data> {
    type Output = Option<Data>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut state = self.0.lock().unwrap();
        if let Some(buf) = state.send_buf.take() {
            Poll::Ready(Some(buf))
        } else if state.num_senders == 0 {
            Poll::Ready(None)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl<Data: ChannelData> Stream for ChannelReceiver<Data> {
    type Item = Data::Item;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        let mut state = self.0.lock().unwrap();

        if let Some(recv_iter) = state.recv_iter.as_mut() {
            if let Some(val) = recv_iter.next() {
                return Poll::Ready(Some(val))
            } else {
                state.recv_iter = None
            }
        }

        if let Some(send_buf) = state.send_buf.take() {
            state.recv_iter = Some(send_buf.into_iter());
            // recv_iter.next() is guaranteed to be Some(x)
            Poll::Ready(state.recv_iter.as_mut().unwrap().next())
        } else if state.num_senders == 0 {
            Poll::Ready(None)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

                    /*\
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
             Factory Functions
<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
                    \*/
pub fn channel<Data: ChannelData>() -> (ChannelSender<Data>, ChannelReceiver<Data>) {
    let state = Arc::new(Mutex::new(ChannelState{
        send_buf: None,
        recv_iter: None,
        num_senders: 1,
        waker: None
    }));

    (ChannelSender(state.clone()), ChannelReceiver(state))
}

pub fn set_channel<T: Eq + Hash + Send + Sync>() -> (ChannelSender<HashSet<T>>, ChannelReceiver<HashSet<T>>) {
    channel::<HashSet<T>>()
}

pub fn queue_channel<T: Send + Sync>() -> (ChannelSender<Vec<T>>, ChannelReceiver<Vec<T>>) {
    channel::<Vec<T>>()
}

pub fn singleton_channel<T: Send + Sync>() -> (ChannelSender<Option<T>>, ChannelReceiver<Option<T>>) {
    channel::<Option<T>>()
}

