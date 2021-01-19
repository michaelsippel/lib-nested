use {
    std::sync::{Arc, RwLock},
    crate::{
        core::{ViewPort, OuterViewPort, InnerViewPort},
        singleton::{SingletonView, SingletonBuffer},
        sequence::VecBuffer
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct StringEditor {
    data: VecBuffer<char>,
    cursor: SingletonBuffer<usize>,

    data_port: ViewPort<RwLock<Vec<char>>>,
    cursor_port: ViewPort<dyn SingletonView<Item = usize>>
}

impl StringEditor {
    pub fn new() -> Self {
        let data_port = ViewPort::new();
        let cursor_port = ViewPort::new();

        StringEditor {
            data: VecBuffer::new(data_port.inner()),
            cursor: SingletonBuffer::new(0, cursor_port.inner()),

            data_port,
            cursor_port
        }
    }

    pub fn get_data_port(&self) -> OuterViewPort<RwLock<Vec<char>>> {
        self.data_port.outer()
    }

    pub fn get_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = usize>> {
        self.cursor_port.outer()
    }

    pub fn goto(&mut self, new_pos: usize) {
        if new_pos <= self.data.len() {
            self.cursor.set(new_pos);
        }
    }

    pub fn goto_end(&mut self) {
        self.cursor.set(self.data.len());
    }

    pub fn prev(&mut self) {
        let cur = self.cursor.get();
        if cur > 0 {
            self.cursor.set(cur - 1);
        }
    }

    pub fn next(&mut self) {
        self.goto(self.cursor.get() + 1);
    }

    pub fn insert(&mut self, c: char) {
        self.data.insert(self.cursor.get(), c);
        self.next();
    }

    pub fn delete(&mut self) {
        let cur = self.cursor.get();
        if cur < self.data.len() {
            self.data.remove(cur);
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod insert_view {
    use {
        std::sync::{Arc, RwLock, Weak},
        cgmath::Point2,
        crate::{
            core::{Observer, ObserverExt, ObserverBroadcast, OuterViewPort, InnerViewPort},
            singleton::SingletonView,
            sequence::SequenceView,
            index::ImplIndexView,
            grid::GridWindowIterator,
            terminal::{TerminalAtom, TerminalStyle, TerminalView}
        }
    };

    struct CursorObserver {
        cursor: Option<Arc<dyn SingletonView<Item = usize>>>,
        edit: Weak<RwLock<StringEditView>>
    }

    impl Observer<dyn SingletonView<Item = usize>> for CursorObserver {
        fn reset(&mut self, new_cursor: Option<Arc<dyn SingletonView<Item = usize>>>) {
            self.cursor = new_cursor;

            if let Some(cursor) = self.cursor.as_ref() {
                self.edit
                    .upgrade().unwrap()
                    .write().unwrap()
                    .update_cursor( cursor.get() );
            }
        }

        fn notify(&self, _msg: &()) {
            if let Some(cursor) = self.cursor.as_ref() {
                self.edit
                    .upgrade().unwrap()
                    .write().unwrap()
                    .update_cursor( cursor.get() );
            }            
        }
    }

    struct DataObserver {
        data: Option<Arc<dyn SequenceView<Item = char>>>,
        edit: Weak<RwLock<StringEditView>>
    }

    impl Observer<dyn SequenceView<Item = char>> for DataObserver {
        fn reset(&mut self, new_data: Option<Arc<dyn SequenceView<Item = char>>>) {
            let old_len = self.data.len().unwrap_or(0);
            self.data = new_data;
            let new_len = self.data.len().unwrap_or(0);

            self.edit
                .upgrade().unwrap()
                .write().unwrap()
                .reset_data( std::cmp::max(old_len, new_len) );
        }

        fn notify(&self, pos: &usize) {
            self.edit
                .upgrade().unwrap()
                .write().unwrap()
                .update_data( *pos );
        }
    }
    
    pub struct StringEditView {
        data_obs: Option<Arc<RwLock<DataObserver>>>,
        cursor_obs: Option<Arc<RwLock<CursorObserver>>>,
        cur_pos: usize,
        cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
    }
    
    impl StringEditView {
        pub fn new(
            cursor_port: OuterViewPort<dyn SingletonView<Item = usize>>,
            data_port: OuterViewPort<dyn SequenceView<Item = char>>,
            out_port: InnerViewPort<dyn TerminalView>
        ) -> Arc<RwLock<Self>> {
            let edit_view = Arc::new(RwLock::new(
                StringEditView {
                    data_obs: None,
                    cursor_obs: None,
                    cur_pos: 0,
                    cast: out_port.get_broadcast()
                }
            ));

            let data_obs = Arc::new(RwLock::new(
                DataObserver {
                    data: None,
                    edit: Arc::downgrade(&edit_view)
                }
            ));
            edit_view.write().unwrap().data_obs = Some(data_obs.clone());
            data_port.add_observer(data_obs);

            let cursor_obs = Arc::new(RwLock::new(
                CursorObserver {
                    cursor: None,
                    edit: Arc::downgrade(&edit_view)
                }
            ));
            edit_view.write().unwrap().cursor_obs = Some(cursor_obs.clone());
            cursor_port.add_observer(cursor_obs);

            out_port.set_view(Some(edit_view.clone()));
            edit_view
        }

        fn reset_data(&mut self, max_len: usize) {
            self.cast.notify_each(GridWindowIterator::from(
                Point2::new(0, 0) .. Point2::new(max_len as i16 +  1, 1)
            ));
        }

        fn update_data(&mut self, pos: usize) {
            self.cast.notify(
                &Point2::new(
                    if pos < self.cur_pos {
                        pos
                    } else {
                        pos + 1
                    } as i16,
                    0
                )
            );
        }

        fn update_cursor(&mut self, new_pos: usize) {
            let old_pos = self.cur_pos;
            self.cur_pos = new_pos;

            self.cast.notify_each(GridWindowIterator::from(
                Point2::new(std::cmp::min(old_pos,new_pos) as i16, 0) .. Point2::new(std::cmp::max(old_pos,new_pos) as i16 + 1, 1)
            ));
        }
    }
    
    impl ImplIndexView for StringEditView {
        type Key = Point2<i16>;
        type Value = TerminalAtom;

        fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
            if pos.y == 0 && pos.x >= 0 {
                let i = pos.x as usize;
                let data =
                    self.data_obs.as_ref().unwrap()
                    .read().unwrap()
                    .data.clone()
                    .unwrap();
                let len = data.len().unwrap();

                if i < len+1 {
                    return Some(
                        if i < self.cur_pos && i < len {
                            TerminalAtom::from(data.get(&i).unwrap())
                        } else if i == self.cur_pos {
                            TerminalAtom::new('|', TerminalStyle::fg_color((200, 0, 0)))
                        } else {
                            TerminalAtom::from(data.get(&(i-1)).unwrap())
                        }
                    );
                }
            }

            None
        }

        fn area(&self) -> Option<Vec<Point2<i16>>> {
            let data =
                self.data_obs.as_ref().unwrap()
                .read().unwrap()
                .data.clone()
                .unwrap();
            let len = data.len()?;

            Some(
                GridWindowIterator::from(
                    Point2::new(0, 0) .. Point2::new(len as i16 + 1, 1)
                ).collect()
            )
        }
    }
}


