
use {
    std::io::{Write, stdout, stdin},
    async_std::stream::{Stream, StreamExt},
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
    Resize((u16, u16)),
    Input(termion::event::Event)
}

pub struct Terminal {
    events: ChannelReceiver<Vec<TerminalEvent>>
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
/*
        let mut resize_stream = signal(tokio::signal::unix::SignalKind::window_change()).unwrap();
        let resize_tx = event_tx.clone();
        tokio::spawn(async move {
            loop {
                resize_stream.recv().await;
                resize_tx.send(TerminalEvent::Resize(termion::terminal_size().unwrap()));
            }
        });
*/
        Terminal {
            events: event_rx
        }
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
                   termion::cursor::Goto(1, 1))?;

        while let Some(atoms) = recv.recv().await {
            for (pos, atom) in atoms.into_iter() {
                if pos != cur_pos {
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
                    write!(out, "{} ", termion::style::Reset);
                }
            }

            out.flush()?;
        }

        write!(out, "{}", termion::cursor::Show)?;
        out.flush()?;

        std::io::Result::Ok(())        
    }
}

