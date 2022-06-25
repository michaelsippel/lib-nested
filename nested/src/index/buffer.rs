use {
    crate::{
        core::{InnerViewPort, OuterViewPort, ViewPort, Observer, ObserverBroadcast, View},
        index::{IndexArea, IndexView},
    },
    std::sync::RwLock,
    std::{collections::HashMap, hash::Hash, sync::Arc, ops::{Deref, DerefMut}},
};

pub struct IndexBufferView<Key, Item>(Arc<RwLock<HashMap<Key, Item>>>)
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static;

impl<Key, Item> View for IndexBufferView<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    type Msg = IndexArea<Key>;
}

impl<Key, Item> IndexView<Key> for IndexBufferView<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    type Item = Item;

    fn get(&self, key: &Key) -> Option<Self::Item> {
        self.0.read().unwrap().get(key).cloned()
    }

    fn area(&self) -> IndexArea<Key> {
        IndexArea::Set(self.0.read().unwrap().keys().cloned().collect())
    }
}

#[derive(Clone)]
pub struct IndexBuffer<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    data: Arc<RwLock<HashMap<Key, Item>>>,
    port: InnerViewPort<dyn IndexView<Key, Item = Item>>,
}

impl<Key, Item> IndexBuffer<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    pub fn with_port(port: InnerViewPort<dyn IndexView<Key, Item = Item>>) -> Self {
        let data = Arc::new(RwLock::new(HashMap::<Key, Item>::new()));
        port.set_view(Some(Arc::new(IndexBufferView(data.clone()))));

        IndexBuffer {
            data,
            port
        }
    }

    pub fn new() -> Self {
        IndexBuffer::with_port(ViewPort::new().into_inner())
    }

    pub fn get_port(&self) -> OuterViewPort<dyn IndexView<Key, Item = Item>> {
        self.port.0.outer()
    }

    pub fn get(&self, key: &Key) -> Option<Item> {
        self.data.read().unwrap().get(key).cloned()
    }

    pub fn get_mut(&mut self, key: &Key) -> MutableIndexAccess<Key, Item> {
        MutableIndexAccess {
            buf: self.clone(),
            key: key.clone(),
            val: self.get(key)
        }
    }

    pub fn update(&mut self, key: Key, item: Option<Item>) {
        if let Some(item) = item {
            self.data.write().unwrap().insert(key.clone(), item);
        } else {
            self.data.write().unwrap().remove(&key);
        }
        self.port.notify(&IndexArea::Set(vec![key]));        
    }
    
    pub fn insert(&mut self, key: Key, item: Item) {
        self.data.write().unwrap().insert(key.clone(), item);
        self.port.notify(&IndexArea::Set(vec![key]));
    }

    pub fn insert_iter<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (Key, Item)>,
    {
        for (key, item) in iter {
            self.insert(key, item);
        }
    }

    pub fn remove(&mut self, key: Key) {
        self.data.write().unwrap().remove(&key);
        self.port.notify(&IndexArea::Set(vec![key]));
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct MutableIndexAccess<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    buf: IndexBuffer<Key, Item>,
    key: Key,
    val: Option<Item>,
}

impl<Key, Item> Deref for MutableIndexAccess<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    type Target = Option<Item>;

    fn deref(&self) -> &Option<Item> {
        &self.val
    }
}

impl<Key, Item> DerefMut for MutableIndexAccess<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<Key, Item> Drop for MutableIndexAccess<Key, Item>
where
    Key: Clone + Hash + Eq + Send + Sync + 'static,
    Item: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.buf.update(self.key.clone(), self.val.clone());
    }
}

