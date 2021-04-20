use {
    std::{
        fs::File,
        os::unix::io::FromRawFd,
        io::{Read, Write, stdout}
    },
    nested::terminal::{
        TerminalAtom,
        TerminalStyle
    },
    termion::raw::IntoRawMode,
    cgmath::{Point2, Vector2}
};

fn main() {
    let mut out = stdout().into_raw_mode().unwrap();
    write!(out, "{}{}{}",
           termion::cursor::Hide,
           termion::cursor::Goto(1, 1),
           termion::style::Reset).unwrap();

    let mut cur_pos = Point2::<i16>::new(0, 0);
    let mut cur_style = TerminalStyle::default();

    let mut f = unsafe { File::from_raw_fd(0) };
    let mut bytes = [0 as u8; 0xe];

    
    let mut input = std::io::stdin();
    let mut buf = [0; 2048];

    loop {
        match bincode::deserialize_from::<_, (Point2<i16>, Option<TerminalAtom>)>(input.by_ref()) {
            Ok((pos, atom)) => {
                if pos != cur_pos {
                    write!(out, "{}", termion::cursor::Goto(pos.x as u16 + 1, pos.y as u16 + 1)).unwrap();
                }

                if let Some(atom) = atom {
                    if cur_style != atom.style {
                        cur_style = atom.style;
                        write!(out, "{}", atom.style);
                    }

                    write!(out, "{}", atom.c.unwrap_or(' '));
                } else {
                    write!(out, "{} ", termion::style::Reset);
                    cur_style = TerminalStyle::default();
                }

                cur_pos = pos + Vector2::new(1, 0);

                out.flush().unwrap();
            }
            Err(err) => {
                match *err {
                    bincode::ErrorKind::Io(io_error) => {
                        break
                    }
                    err => {
                        eprintln!("deserialization error: {:?}\n{:?}", bytes, err);
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

