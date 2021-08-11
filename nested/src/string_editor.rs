use {
    std::sync::RwLock,
    termion::event::{Key, Event},
    crate::{
        core::{ViewPort, OuterViewPort},
        singleton::{SingletonView, SingletonBuffer},
        vec::VecBuffer,
        terminal::{TerminalView, TerminalEvent, TerminalEditor, TerminalEditorResult}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone)]
pub struct StringEditor {
    data: VecBuffer<char>,
    cursor: SingletonBuffer<Option<usize>>,

    data_port: ViewPort<RwLock<Vec<char>>>,
    cursor_port: ViewPort<dyn SingletonView<Item = Option<usize>>>
}

impl StringEditor {
    pub fn new() -> Self {
        let data_port = ViewPort::new();
        let cursor_port = ViewPort::new();

        StringEditor {
            data: VecBuffer::new(data_port.inner()),
            cursor: SingletonBuffer::new(None, cursor_port.inner()),

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

    pub fn get_cursor_port(&self) -> OuterViewPort<dyn SingletonView<Item = Option<usize>>> {
        self.cursor_port.outer()
    }

    pub fn goto(&mut self, new_pos: usize) -> TerminalEditorResult {
        if new_pos <= self.data.len() {
            self.cursor.set(Some(new_pos));
            TerminalEditorResult::Continue
        } else {
            self.cursor.set(None);
            TerminalEditorResult::Exit
        }
    }

    pub fn goto_end(&mut self) -> TerminalEditorResult {
        self.goto(self.data.len())
    }

    pub fn prev(&mut self) -> TerminalEditorResult {
        let cur = self.cursor.get().unwrap_or(usize::MAX);
        if cur > 0 {
            self.cursor.set(Some(cur - 1));
            TerminalEditorResult::Continue
        } else {
            self.cursor.set(None);
            TerminalEditorResult::Exit
        }
    }

    pub fn next(&mut self) -> TerminalEditorResult {
        self.goto(self.cursor.get().unwrap_or(0) + 1)
    }

    pub fn insert(&mut self, c: char) -> TerminalEditorResult {
        self.data.insert(self.cursor.get().unwrap_or(0), c);
        self.next()
    }

    pub fn delete_prev(&mut self) -> TerminalEditorResult {
        let cur = self.cursor.get().unwrap_or(0);
        if cur <= self.data.len() && cur > 0 {
            self.data.remove(cur-1);
        }
        self.prev()
    }

    pub fn delete(&mut self) -> TerminalEditorResult {
        let cur = self.cursor.get().unwrap_or(0);
        if cur < self.data.len() {
            self.data.remove(cur);
            TerminalEditorResult::Continue
        } else {
            self.cursor.set(None);
            TerminalEditorResult::Exit
        }
    }
}

impl TerminalEditor for StringEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.insert_view()
    }

    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult {
        match event {
            TerminalEvent::Input(Event::Key(Key::Left)) => self.prev(),
            TerminalEvent::Input(Event::Key(Key::Right)) => self.next(),

            TerminalEvent::Input(Event::Key(Key::Up)) => {
                self.cursor.set(None);
                TerminalEditorResult::Exit
            }
            
            TerminalEvent::Input(Event::Key(Key::Down)) |
            TerminalEvent::Input(Event::Key(Key::Home)) =>
                if self.cursor.get() == Some(0) {
                    self.cursor.set(None);
                    TerminalEditorResult::Exit
                } else {
                    self.goto(0)
                },
            TerminalEvent::Input(Event::Key(Key::End)) =>
                if self.cursor.get() == Some(self.data.len()) {
                    self.cursor.set(None);
                    TerminalEditorResult::Exit
                } else {
                    self.goto_end()
                },
            TerminalEvent::Input(Event::Key(Key::Char('\n'))) => TerminalEditorResult::Continue,
            TerminalEvent::Input(Event::Key(Key::Char(c))) => self.insert(*c),
            TerminalEvent::Input(Event::Key(Key::Delete)) => self.delete(),
            TerminalEvent::Input(Event::Key(Key::Backspace)) => self.delete_prev(),
            _ => TerminalEditorResult::Continue
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod insert_view {
    use {
        std::{
            sync::Arc,
            cmp::{min, max}
        },
        cgmath::Point2,
        std::sync::RwLock,
        crate::{
            core::{View, Observer, ObserverExt, ObserverBroadcast, OuterViewPort, InnerViewPort},
            terminal::{TerminalAtom, TerminalStyle, TerminalView},
            grid::{GridWindowIterator},
            singleton::{SingletonView},
            sequence::{SequenceView},
            index::{IndexView},
            projection::{ProjectionHelper},
        }
    };

    pub struct StringInsertView {
        cursor: Arc<dyn SingletonView<Item = Option<usize>>>,
        data: Arc<RwLock<dyn SequenceView<Item = char>>>,
        cur_pos: Option<usize>,

        cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
        proj_helper: ProjectionHelper<Self>
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
                        if i < self.cur_pos.unwrap_or(usize::MAX) {
                            TerminalAtom::from(data.get(&i)?)
                        } else if i == self.cur_pos.unwrap_or(usize::MAX) {
                            TerminalAtom::new('|', TerminalStyle::fg_color((90, 60, 200)))
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
                Point2::new(0, 0) .. Point2::new(self.data.len()? as i16 + if self.cursor.get().is_some() { 1 } else { 0 }, 1)
            ).collect())
        }
    }

    impl StringInsertView {
        pub fn new(
            cursor_port: OuterViewPort<dyn SingletonView<Item = Option<usize>>>,
            data_port: OuterViewPort<dyn SequenceView<Item = char>>,
            out_port: InnerViewPort<dyn TerminalView>
        ) -> Arc<RwLock<Self>> {
            let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());

            let proj = Arc::new(RwLock::new(
                StringInsertView {
                    cursor: proj_helper.new_singleton_arg(
                        cursor_port,
                        |s: &mut Self, _msg| {
                            let old_pos = s.cur_pos.unwrap_or(0);
                            s.cur_pos = s.cursor.get();
                            let new_pos = s.cur_pos.unwrap_or(0);
                            s.cast.notify_each(GridWindowIterator::from(Point2::new(min(old_pos, new_pos) as i16,0) ..= Point2::new(max(old_pos, new_pos) as i16, 0)))
                        }),

                    data: proj_helper.new_sequence_arg(
                        data_port,
                        |s: &mut Self, idx| {
                            s.cast.notify(&Point2::new(
                                if *idx < s.cur_pos.unwrap_or(0) {
                                    *idx as i16
                                } else {
                                    *idx as i16 + 1
                                },
                                0
                            ));
                        }),

                    cur_pos: None,
                    cast: out_port.get_broadcast(),

                    proj_helper
                }
            ));

            proj.write().unwrap().proj_helper.set_proj(&proj);
            out_port.set_view(Some(proj.clone()));

            proj
        }
    }
}


