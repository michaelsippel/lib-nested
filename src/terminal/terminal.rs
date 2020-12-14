
use {
    std::io::{Write, stdout, stdin},
    async_std::{
        stream::StreamExt,
        task
    },
    signal_hook,
    signal_hook_async_std::Signals,
    cgmath::Vector2,
    termion::{
        raw::IntoRawMode,
        input::{TermRead, MouseTerminal}
    },
    super::{TerminalAtom, TerminalStyle},
    crate::{
        view::{View, Observer},
        port::{OuterViewPort},
        channel::ChannelReceiver
    }
};

pub enum TerminalEvent {
    Resize(Vector2<i16>),
    Input(termion::event::Event)
}

pub struct Terminal {
    events: ChannelReceiver<Vec<TerminalEvent>>,
    signal_handle: signal_hook_async_std::Handle
}

impl Terminal {
    pub fn new() -> Self {
        let (event_tx, event_rx) = crate::channel::queue_channel();

        let input_tx = event_tx.clone();
        std::thread::spawn(move || {
            for event in stdin().events() {
                input_tx.notify(TerminalEvent::Input(event.unwrap()));
            }
        });

        // send initial teriminal size
        let (w,h) = termion::terminal_size().unwrap();
        event_tx.notify(TerminalEvent::Resize(Vector2::new(w as i16, h as i16)));

        // and again on SIGWINCH
        let signals = Signals::new(&[ signal_hook::SIGWINCH ]).unwrap();
        let handle = signals.handle();

        task::spawn(async move {
            let mut signals = signals.fuse();
            while let Some(signal) = signals.next().await {
                match signal {
                    signal_hook::SIGWINCH => {
                        let (w,h) = termion::terminal_size().unwrap();
                        event_tx.notify(TerminalEvent::Resize(Vector2::new(w as i16, h as i16)));
                    },
                    _ => unreachable!(),
                }
            }
        });

        Terminal {
            events: event_rx,
            signal_handle: handle
        }
    }

    pub async fn next_event(&mut self) -> TerminalEvent {
        self.events.next().await.unwrap()
    }

    pub async fn show(view_port: OuterViewPort<Vector2<i16>, TerminalAtom>) -> std::io::Result<()> {
        let (atom_tx, atom_rx) = crate::channel::queue_channel();

        let view = view_port.get_view();
        view_port.add_observer_fn(move |pos| atom_tx.notify((pos, view.view(pos))));

        Self::show_stream(atom_rx).await
    }

    pub async fn show_stream(recv: ChannelReceiver<Vec<(Vector2<i16>, Option<TerminalAtom>)>>) -> std::io::Result<()> {
        let mut out = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        let mut cur_pos = Vector2::<i16>::new(0, 0);
        let mut cur_style = TerminalStyle::default();
            write!(out, "{}{}{}{}",
                   termion::clear::All,
                   termion::cursor::Goto(1, 1),
                   termion::cursor::Hide,
                   termion::style::Reset)?;

        while let Some(atoms) = recv.recv().await {
            for (pos, atom) in atoms.into_iter() {
                if pos != cur_pos+Vector2::new(1,0) {
                    write!(out, "{}", termion::cursor::Goto(pos.x as u16 + 1, pos.y as u16 + 1))?;
                }
                cur_pos = pos;

                if let Some(atom) = atom {
                    if cur_style != atom.style {
                        cur_style = atom.style;
                        write!(out, "{}", atom.style)?;
                    }

                    write!(out, "{}", atom.c.unwrap_or(' '))?;
                } else {
                    write!(out, "{} ", termion::style::Reset)?;
                    cur_style = TerminalStyle::default();
                }
            }

            out.flush()?;
        }

        write!(out, "{}", termion::cursor::Show)?;
        out.flush()?;

        std::io::Result::Ok(())        
    }
}

