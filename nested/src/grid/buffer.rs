
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


struct GridBuffer<Item> {
    data: HashMap<Point2<i16>, Item>,
    limit: Point2<i16>
}

impl<Item> View for GridBuffer<Item>
where Item: Clone + Send + Sync + 'static
{
    type Msg = IndexArea<Point2<i16>>;
}

impl<Item> IndexView<Point2<i16>> for GridBufferView<Item>
where Item: Clone + Send + Sync + 'static
{
    type Item = Item;

    fn get(&self, key: &Point2<i16>) -> Option<Self::Item> {
        self.data.get(key).cloned()
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(
            Point2::new(0, 0)
                ..= self.limit
        )
    }
}

pub struct GridBufferController<Item>
where Item: Clone + Send + Sync + 'static
{
    data: Arc<RwLock<HashMap<Point2<i16>, Item>>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn IndexView<Point2<i16>, Item = Item>>>>
}

impl<Key, Item> GridBuffer<Key, Item>
where Key: Clone + Hash + Eq + Send + Sync + 'static,
      Item: Clone + Send + Sync + 'static
{
    pub fn new(port: InnerViewPort<dyn IndexView<Point2<i16>, Item = Item>>) -> Self {
        let data = Arc::new(RwLock::new(HashMap::<Point2<i16>, Item>::new()));
        port.set_view(Some(Arc::new(GridBufferView(data.clone()))));

        GridBuffer {
            data,
            cast: port.get_broadcast()
        }
    }

    pub fn insert(&mut self, key: Point2<i16>, item: Item) {
        self.data.write().unwrap().insert(key.clone(), item);

        if
        
        self.cast.notify(&IndexArea::Set(vec![ key ]));
    }

    pub fn insert_iter<T>(&mut self, iter: T)
    where T: IntoIterator<Item = (Point2<i16>, Item)> {
        for (key, item) in iter {
            self.insert(key, item);
        }
    }

    pub fn remove(&mut self, key: Point2<i16>) {
        self.data.write().unwrap().remove(&key);
        self.cast.notify(&IndexArea::Set(vec![ key ]));
    }
}

