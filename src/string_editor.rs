use {
    std::sync::{Arc, RwLock},
    cgmath::Vector2,
    crate::{
        view::{View, Observer, ObserverExt},
        port::{ViewPort, InnerViewPort, OuterViewPort},
        terminal::{TerminalAtom, TerminalStyle},
        vec_buffer::VecBuffer
    }
};

pub struct StringEditorState {
    cursor: usize,
    data: Arc<RwLock<Vec<char>>>
}

impl View for StringEditorState {
    type Key = Vector2<i16>;
    type Value = TerminalAtom;

    fn view(&self, pos: Vector2<i16>) -> Option<TerminalAtom> {
        if pos.y == 0 {
            let cur = self.cursor;
            let data = self.data.read().unwrap();

            if pos.x < data.len() as i16 + 3 {
                let i = pos.x as usize;
                return Some(
                    if i == 0 {
                        TerminalAtom::new('"', TerminalStyle::fg_color((180,200,130)))
                    } else if i-1 == cur {
                        TerminalAtom::new('|', TerminalStyle::fg_color((180,200,130)).add(TerminalStyle::bold(true)))
                    } else if i-1 == data.len()+1 {
                        TerminalAtom::new('"', TerminalStyle::fg_color((180,200,130)))
                    } else {
                        TerminalAtom::new(
                            data.get(i as usize - if i <= cur { 1 } else { 2 }).cloned().unwrap(),
                            TerminalStyle::fg_color((80,150,80))
                        )
                    }
                )
            }
        }

        None
    }
}

pub struct StringEditor {
    state: Arc<RwLock<StringEditorState>>,
    port: InnerViewPort<Vector2<i16>, TerminalAtom>
}

impl StringEditor {
    pub fn new(
        port: InnerViewPort<Vector2<i16>, TerminalAtom>
    ) -> Self {
        let state = Arc::new(RwLock::new(StringEditorState{
            cursor: 0,
            data: Arc::new(RwLock::new(Vec::new()))
        }));
/*
        let buf_port = ViewPort::new();
        let buf = VecBuffer::with_data(data.clone(), buf_port.inner());

        buf_port.outer().add_observer_fn({
            let port = port.clone();
            let cursor = cursor.clone();

            move |idx|
            if idx < *cursor.read().unwrap() {
                port.notify(Vector2::new(1 + idx as i16, 0));
            } else {
                port.notify(Vector2::new(2 + idx as i16, 0));
            }
        });
*/
        port.set_view(state.clone());
        StringEditor {
            state,
            port
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

        self.port.notify_each(
            (std::cmp::min(old_idx, new_idx) ..= std::cmp::max(old_idx, new_idx))
            .map(|idx| Vector2::new(1+idx as i16, 0))
        );
    }

    pub fn insert(&mut self, c: char) {
        self.port.notify_each({
            let mut state = self.state.write().unwrap();
            let mut data = state.data.write().unwrap();

            data.insert(state.cursor, c);
            (state.cursor .. data.len()+2)
        }.map(|idx| Vector2::new(1+idx as i16, 0)));

        self.next();
    }

    pub fn delete(&mut self) {
        self.port.notify_each({
            let mut state = self.state.write().unwrap();
            let mut data = state.data.write().unwrap();

            if state.cursor < data.len() {
                data.remove(state.cursor);
            }
            (state.cursor .. data.len()+3)
        }.map(|idx| Vector2::new(1+idx as i16, 0)));
    }
}

