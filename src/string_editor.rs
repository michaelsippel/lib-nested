
use {
    std::{
        sync::{Arc, RwLock},
    },
    cgmath::Point2,
    crate::{
        core::{
            ObserverExt,
            ObserverBroadcast,
            InnerViewPort
        },
        index::{ImplIndexView},
        grid::{GridWindowIterator},
        terminal::{TerminalAtom, TerminalStyle, TerminalView},
        //vec_buffer::VecBuffer
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct StringEditorState {
    cursor: usize,
    data: Arc<RwLock<Vec<char>>>
}

impl ImplIndexView for StringEditorState {
    type Key = Point2<i16>;
    type Value = Option<TerminalAtom>;

    fn get(&self, pos: &Point2<i16>) -> Option<TerminalAtom> {
        let data = self.data.read().unwrap();

        if pos.y == 0 {
            let i = pos.x as usize;
            if i < data.len() + 3 {
                return Some(
                    if i == 0 {
                        TerminalAtom::new('"', TerminalStyle::fg_color((180,200,130)))
                    } else if i-1 == self.cursor {
                        TerminalAtom::new('|', TerminalStyle::fg_color((180,200,130)).add(TerminalStyle::bold(false)))
                    } else if i-1 == data.len()+1 {
                        TerminalAtom::new('"', TerminalStyle::fg_color((180,200,130)))
                    } else {
                        TerminalAtom::new(
                            data.get(i as usize - if i <= self.cursor { 1 } else { 2 }).unwrap().clone(),
                            TerminalStyle::fg_color((80,150,80)).add(TerminalStyle::bold(true))
                        )
                    }
                )
            }
        }

        None
    }        

    fn area(&self) -> Option<Vec<Point2<i16>>> {
        Some(GridWindowIterator::from(
            Point2::new(0, 0)
                .. Point2::new(self.data.read().unwrap().len() as i16 + 3, 1)).collect())
    }
}

pub struct StringEditor {
    state: Arc<RwLock<StringEditorState>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>
}

impl StringEditor {
    pub fn new(
        port: InnerViewPort<dyn TerminalView>
    ) -> Self {
        let state = Arc::new(RwLock::new(StringEditorState{
            cursor: 7,
            data: Arc::new(RwLock::new("edit me".chars().collect()))
        }));

        let cast = port.set_view(Some(state.clone()));

        StringEditor {
            state,
            cast
        }
    }

    pub fn next(&mut self) {
        let cur = self.state.read().unwrap().cursor;
        self.goto(cur + 1);
    }

    pub fn prev(&mut self) {
        let cur = self.state.read().unwrap().cursor;
        if cur > 0 {
            self.goto(cur - 1);
        }
    }

    pub fn goto_end(&mut self) {
        let l = self.state.read().unwrap().data.read().unwrap().len();
        self.goto(l);
    }

    pub fn goto(&mut self, mut new_idx: usize) {
        let old_idx = {
            let mut state = self.state.write().unwrap();
            let old_idx = state.cursor.clone();
            let len = state.data.read().unwrap().len();
            new_idx = std::cmp::min(new_idx, len);
            state.cursor = new_idx;
            old_idx
        };

        self.cast.notify_each(
            (std::cmp::min(old_idx, new_idx) ..= std::cmp::max(old_idx, new_idx))
            .map(|idx| Point2::new(1+idx as i16, 0))
        );
    }

    pub fn insert(&mut self, c: char) {
        self.cast.notify_each({
            let state = self.state.write().unwrap();
            let mut data = state.data.write().unwrap();

            data.insert(state.cursor, c);

            state.cursor .. data.len()+2
        }.map(|idx| Point2::new(1+idx as i16, 0)));

        self.next();
    }

    pub fn delete(&mut self) {
        self.cast.notify_each({
            let state = self.state.write().unwrap();
            let mut data = state.data.write().unwrap();

            if state.cursor < data.len() {
                data.remove(state.cursor);
            }

            state.cursor .. data.len()+3
        }.map(|idx| Point2::new(1+idx as i16, 0)));
    }
}

