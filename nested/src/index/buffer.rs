
use {
    std::{
        sync::Arc,
        collections::HashMap,
        hash::Hash
    },
    std::sync::RwLock,
    crate::{
        core::{
            Observer,
            ObserverBroadcast,
            View,
            InnerViewPort
        },
        index::{IndexArea, IndexView}
    }
};

pub struct IndexBufferView<Key, Item>(Arc<RwLock<HashMap<Key, Item>>>)
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static;

impl<Key, Item> View for IndexBufferView<Key, Item>
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static
{
    type Msg = IndexArea<Key>;
}

impl<Key, Item> IndexView<Key> for IndexBufferView<Key, Item>
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static
{
    type Item = Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.0.read().unwrap().get(key).cloned()
    }

    fn area(&self) -> IndexArea<Key> {
        IndexArea::Set(self.0.read().unwrap().keys().cloned().collect())
    }
}

pub struct IndexBuffer<Key, Item>
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static
{
    data: Arc<RwLock<HashMap<Key, Item>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<Key, Item = Item>>>>
}

impl<Key, Item> IndexBuffer<Key, Item>
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static
{
    pub fn new(port: InnerViewPort<dyn IndexView<Key, Item = Item>>) -> Self {
        let data = Arc::new(RwLock::new(HashMap::<Key, Item>::new()));
        port.set_view(Some(Arc::new(IndexBufferView(data.clone()))));

        IndexBuffer {
            data,
            cast: port.get_broadcast()
        }
    }

    pub fn insert(&mut self, key: Key, item: Item) {
        self.data.write().unwrap().insert(key.clone(), item);
        self.cast.notify(&IndexArea::Set(vec![ key ]));
    }

    pub fn insert_iter<T>(&mut self, iter: T)
    where T: IntoIterator<Item = (Key, Item)> {
        for (key, item) in iter {
            self.insert(key, item);
        }
    }

    pub fn remove(&mut self, key: Key) {
        self.data.write().unwrap().remove(&key);
        self.cast.notify(&IndexArea::Set(vec![ key ]));
    }
}

