use {
    r3vi::{
        view::{
            channel::{queue_channel, set_channel, ChannelReceiver, ChannelSender},
            Observer, OuterViewPort,
            grid::*,
            index::*,
        }
    },
    super::{TerminalStyle, TerminalView},
    async_std::{stream::StreamExt, task},
    cgmath::{Point2, Vector2},
    signal_hook,
    signal_hook_async_std::Signals,
    std::sync::RwLock,
    std::{
        collections::HashSet,
        io::{stdin, stdout, Write},
        sync::Arc,
    },
    termion::{
        input::{MouseTerminal, TermRead},
        raw::IntoRawMode,
    },
};

#[derive(PartialEq, Eq, Clone)]
pub enum TerminalEvent {
    Resize(Vector2<i16>),
    Input(termion::event::Event),
}

pub struct Terminal {
    writer: Arc<TermOutWriter>,
    _observer: Arc<RwLock<TermOutObserver>>,

    events: ChannelReceiver<Vec<TerminalEvent>>,
    _signal_handle: signal_hook_async_std::Handle,
}

impl Terminal {
    pub fn new(port: OuterViewPort<dyn TerminalView>) -> Self {
        let (dirty_pos_tx, dirty_pos_rx) = set_channel();

        let writer = Arc::new(TermOutWriter {
            out: RwLock::new(MouseTerminal::from(stdout().into_raw_mode().unwrap())),
            dirty_pos_rx,
            view: port.get_view_arc(),
        });

        let observer = Arc::new(RwLock::new(TermOutObserver {
            dirty_pos_tx,
            writer: writer.clone(),
        }));

        port.add_observer(observer.clone());

        let (event_tx, event_rx) = queue_channel();

        let input_tx = event_tx.clone();
        std::thread::spawn(move || {
            for event in stdin().events() {
                input_tx.send(TerminalEvent::Input(event.unwrap()));
            }
        });

        // send initial teriminal size
        let (w, h) = termion::terminal_size().unwrap();
        event_tx.send(TerminalEvent::Resize(Vector2::new(w as i16, h as i16)));

        // and again on SIGWINCH
        let signals = Signals::new(&[signal_hook::consts::signal::SIGWINCH]).unwrap();
        let handle = signals.handle();

        task::spawn(async move {
            let mut signals = signals.fuse();
            while let Some(signal) = signals.next().await {
                match signal {
                    signal_hook::consts::signal::SIGWINCH => {
                        let (w, h) = termion::terminal_size().unwrap();
                        event_tx.send(TerminalEvent::Resize(Vector2::new(w as i16, h as i16)));
                    }
                    _ => unreachable!(),
                }
            }
        });

        Terminal {
            writer,
            _observer: observer,
            events: event_rx,
            _signal_handle: handle,
        }
    }

    pub fn get_writer(&self) -> Arc<TermOutWriter> {
        self.writer.clone()
    }

    pub async fn next_event(&mut self) -> TerminalEvent {
        self.events.next().await.unwrap()
    }
}

struct TermOutObserver {
    dirty_pos_tx: ChannelSender<HashSet<Point2<i16>>>,
    writer: Arc<TermOutWriter>,
}

impl TermOutObserver {
    fn send_area(&mut self, area: IndexArea<Point2<i16>>) {
        match area {
            IndexArea::Empty => {}
            IndexArea::Full => {
                let (w, h) = termion::terminal_size().unwrap();
                for pos in
                    GridWindowIterator::from(Point2::new(0, 0)..Point2::new(w as i16, h as i16))
                {
                    self.dirty_pos_tx.send(pos);
                }
            }
            IndexArea::Range(r) => {
                for pos in GridWindowIterator::from(r) {
                    self.dirty_pos_tx.send(pos);
                }
            }
            IndexArea::Set(v) => {
                for pos in v {
                    self.dirty_pos_tx.send(pos);
                }
            }
        }
    }
}

impl Observer<dyn TerminalView> for TermOutObserver {
    fn reset(&mut self, view: Option<Arc<dyn TerminalView>>) {
        self.writer.reset();
        if let Some(view) = view {
            self.send_area(view.area());
        }
    }

    fn notify(&mut self, area: &IndexArea<Point2<i16>>) {
        self.send_area(area.clone());
    }
}

pub struct TermOutWriter {
    out: RwLock<MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>,
    dirty_pos_rx: ChannelReceiver<HashSet<Point2<i16>>>,
    view: Arc<RwLock<Option<Arc<dyn TerminalView>>>>,
}

impl TermOutWriter {
    fn reset(&self) {
        let mut out = self.out.write().unwrap();
        write!(out, "{}", termion::clear::All).ok();
    }
}

impl TermOutWriter {
    pub async fn show(&self) -> std::io::Result<()> {
        // init
        write!(
            self.out.write().unwrap(),
            "{}{}{}",
            termion::cursor::Hide,
            termion::cursor::Goto(1, 1),
            termion::style::Reset
        )?;

        let mut cur_pos = Point2::<i16>::new(0, 0);
        let mut cur_style = TerminalStyle::default();

        // draw atoms until view port is destroyed
        while let Some(dirty_pos) = self.dirty_pos_rx.recv().await {
            let (w, _h) = termion::terminal_size().unwrap();

            if let Some(view) = self.view.read().unwrap().as_ref() {
                let mut out = self.out.write().unwrap();

                let d = dirty_pos
                    .into_iter()
                    .filter(|p| p.x >= 0 && p.y >= 0 && p.x < w as i16 && p.y < w as i16); //.collect::<Vec<_>>();
                                                                                           /*
                                                                                                           d.sort_by(|a,b| {
                                                                                                               if a.y < b.y {
                                                                                                                   std::cmp::Ordering::Less
                                                                                                               } else if a.y == b.y {
                                                                                                                   a.x.cmp(&b.x)
                                                                                                               } else {
                                                                                                                   std::cmp::Ordering::Greater
                                                                                                               }
                                                                                                           });
                                                                                           */
                for pos in d {
                    if pos != cur_pos {
                        write!(
                            out,
                            "{}",
                            termion::cursor::Goto(pos.x as u16 + 1, pos.y as u16 + 1)
                        )?;
                    }

                    if let Some(atom) = view.get(&pos) {
                        if cur_style != atom.style {
                            cur_style = atom.style;
                            write!(out, "{}", termion::style::Reset)?;
                            write!(out, "{}", atom.style)?;
                        }

                        write!(out, "{}", atom.c.unwrap_or(' '))?;
                    } else {
                        write!(out, "{} ", termion::style::Reset)?;
                        cur_style = TerminalStyle::default();
                    }

                    cur_pos = pos + Vector2::new(1, 0);
                }

                out.flush()?;
            }
        }

        // restore conventional terminal settings
        let mut out = self.out.write().unwrap();
        write!(out, "{}", termion::cursor::Show)?;
        out.flush()?;

        std::io::Result::Ok(())
    }
}
