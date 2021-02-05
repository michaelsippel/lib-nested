use {
    std::{
        sync::{RwLock}
    },
    crate::{
        core::{ViewPort, OuterViewPort},
        singleton::{SingletonView, SingletonBuffer},
        sequence::VecBuffer,
        terminal::{TerminalView}
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

    pub fn insert_view(&self) -> OuterViewPort<dyn TerminalView> {
        let port = ViewPort::new();
        insert_view::StringInsertView::new(
            self.get_cursor_port(),
            self.get_data_port().to_sequence(),
            port.inner()
        );

        port.into_outer()
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

    pub fn delete_prev(&mut self) {
        let cur = self.cursor.get();
        if cur <= self.data.len() && cur > 0 {
            self.data.remove(cur-1);
        }
        self.prev();
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
        std::{
            sync::{Arc, RwLock},
            cmp::{min, max},
            any::Any,
            collections::HashSet
        },
        cgmath::Point2,
        crate::{
            core::{View, Observer, ObserverExt, ObserverBroadcast, OuterViewPort, InnerViewPort},
            terminal::{TerminalAtom, TerminalStyle, TerminalView},
            grid::{GridWindowIterator},
            singleton::{SingletonView},
            sequence::{SequenceView},
            index::{IndexView},
            projection::{ProjectionHelper, ProjectionArg},
        }
    };

    pub struct StringInsertView {
        proj_helper: Option<ProjectionHelper<Self>>,

        cursor: Arc<dyn SingletonView<Item = usize>>,
        data: Arc<RwLock<dyn SequenceView<Item = char>>>,
        cur_pos: usize,
        cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
    }

    impl View for StringInsertView {
        type Msg = Point2<i16>;
    }

    impl IndexView<Point2<i16>> for StringInsertView {
        type Item = TerminalAtom;

        fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
            if pos.y == 0 && pos.x >= 0 {
                let i = pos.x as usize;
                let data = self.data.read().unwrap();
                let len = data.len().unwrap_or(0);

                if i < len+1 {
                    return Some(
                        if i < self.cur_pos {
                            TerminalAtom::from(data.get(&i)?)
                        } else if i == self.cur_pos {
                            TerminalAtom::new('|', TerminalStyle::fg_color((200, 0, 0)))
                        } else {
                            TerminalAtom::from(data.get(&(i - 1))?)
                        }
                    );
                }
            }

            None
        }

        fn area(&self) -> Option<Vec<Point2<i16>>> {
            Some(GridWindowIterator::from(
                Point2::new(0, 0) .. Point2::new(self.data.len()? as i16 + 1, 1)
            ).collect())
        }
    }

    impl StringInsertView {
        pub fn new(
            cursor_port: OuterViewPort<dyn SingletonView<Item = usize>>,
            data_port: OuterViewPort<dyn SequenceView<Item = char>>,
            out_port: InnerViewPort<dyn TerminalView>
        ) -> Arc<RwLock<Self>> {
            let proj = Arc::new(RwLock::new(
                StringInsertView {
                    proj_helper: None,
                    cursor: Arc::new(Option::<Arc<dyn SingletonView<Item = usize>>>::None),
                    data: Arc::new(RwLock::new(Option::<Arc<dyn SequenceView<Item = char>>>::None)),
                    cur_pos: 0,
                    cast: out_port.get_broadcast()
                }
            ));

            let mut projection_helper = ProjectionHelper::new(Arc::downgrade(&proj));

            let (cursor, cursor_obs) = projection_helper.new_arg(
                |s: Arc<RwLock<Self>>, _msg| {
                    let old_pos = s.read().unwrap().cur_pos;
                    let new_pos = s.read().unwrap().cursor.get();
                    s.write().unwrap().cur_pos = new_pos;
                    s.read().unwrap().cast.notify_each(GridWindowIterator::from(Point2::new(min(old_pos, new_pos) as i16,0) ..= Point2::new(max(old_pos, new_pos) as i16, 0)))
                });

            let (data, data_obs) = projection_helper.new_arg(
                |s: Arc<RwLock<Self>>, idx| {
                    s.read().unwrap().cast.notify(&Point2::new(
                        if *idx < s.read().unwrap().cur_pos {
                            *idx as i16
                        } else {
                            *idx as i16 + 1
                        },
                        0
                    ));
                });

            proj.write().unwrap().proj_helper = Some(projection_helper);
            proj.write().unwrap().cursor = cursor;
            proj.write().unwrap().data = data;

            cursor_port.add_observer(cursor_obs);
            data_port.add_observer(data_obs);

            out_port.set_view(Some(proj.clone()));

            proj
        }
    }
}


