use {
    cgmath::{Point2, Vector2},
    nested::terminal::{TerminalAtom, TerminalStyle},
    std::io::{stdout, Read, Write},
    termion::raw::IntoRawMode,
};

fn main() {
    let mut out = stdout().into_raw_mode().unwrap();
    write!(
        out,
        "{}{}{}",
        termion::cursor::Hide,
        termion::cursor::Goto(1, 1),
        termion::style::Reset
    )
    .unwrap();

    let mut cur_pos = Point2::<i16>::new(0, 0);
    let mut cur_style = TerminalStyle::default();

    let mut input = std::io::stdin();

    loop {
        match bincode::deserialize_from::<_, (Point2<i16>, Option<TerminalAtom>)>(input.by_ref()) {
            Ok((pos, atom)) => {
                if pos != cur_pos {
                    write!(
                        out,
                        "{}",
                        termion::cursor::Goto(pos.x as u16 + 1, pos.y as u16 + 1)
                    )
                    .unwrap();
                }

                if let Some(atom) = atom {
                    if cur_style != atom.style {
                        cur_style = atom.style;
                        write!(out, "{}", atom.style).expect("");
                    }

                    write!(out, "{}", atom.c.unwrap_or(' ')).expect("");
                } else {
                    write!(out, "{} ", termion::style::Reset).expect("");
                    cur_style = TerminalStyle::default();
                }

                cur_pos = pos + Vector2::new(1, 0);

                out.flush().unwrap();
            }
            Err(err) => {
                match *err {
                    bincode::ErrorKind::Io(_io_error) => break,
                    err => {
                        eprintln!("deserialization error\n{:?}", err);
                    }
                }
                break;
            }
        }
    }

    // restore conventional terminal settings
    write!(out, "{}", termion::cursor::Show).unwrap();
    out.flush().unwrap();
}
