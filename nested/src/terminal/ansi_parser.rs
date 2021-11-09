use {
    std::{
        sync::{Arc, RwLock},
        pin::Pin,
        fs::File,
        os::unix::io::FromRawFd
    },
    std::io::Read,
    //async_std::{io::{Read, ReadExt}},
    crate::{
        core::{InnerViewPort, OuterViewPort},
        terminal::{
            TerminalAtom,
            TerminalStyle,
            TerminalView
        },
        index::buffer::IndexBuffer
    },
    cgmath::Point2,
    vte::{Params, Parser, Perform}
};

pub fn read_ansi_from<R: Read + Unpin>(ansi_reader: &mut R, port: InnerViewPort<dyn TerminalView>) {
    let mut statemachine = Parser::new();

    let mut performer = PerfAtom {
        cursor: Point2::new(0, 0),
        style: TerminalStyle::default(),
        invert: false,
        term_width: 80,

        cursor_save: Point2::new(0, 0),

        buf: IndexBuffer::new(port),

        colors: ColorPalett {
            black: (1, 1, 1),
            red: (222, 56, 43),
            green: (0, 64, 0),
            yellow: (255, 199, 6),
            blue: (0, 111, 184),
            magenta: (118, 38, 113),
            cyan: (44, 181, 233),
            white: (204, 204, 204)
        }
    };

    let mut buf = [0; 2048];

    loop {
        match ansi_reader.read(&mut buf) {
            //Ok(0) => break,
            Ok(n) => {
                for byte in &buf[..n] {
                    statemachine.advance(&mut performer, *byte);
                }
            },
            Err(err) => {
                //println!("err: {}", err);
                break;
            },
        }
    }
}

struct ColorPalett {
    black: (u8, u8, u8),
    red: (u8, u8, u8),
    green: (u8, u8, u8),
    yellow: (u8, u8, u8),
    blue: (u8, u8, u8),
    magenta: (u8, u8, u8),
    cyan: (u8, u8, u8),
    white: (u8, u8, u8)
}

struct PerfAtom {
    colors: ColorPalett,
    term_width: i16,

    cursor: Point2<i16>,
    style: TerminalStyle,
    invert: bool,
    cursor_save: Point2<i16>,

    buf: IndexBuffer<Point2<i16>, TerminalAtom>,
}

impl PerfAtom {
    fn write_atom(&mut self, pos: Point2<i16>, atom: Option<TerminalAtom>) {
        if let Some(mut a) = atom {
            self.buf.insert(pos, a);
        } else {
            self.buf.remove(pos);
        }
    }

    fn get_style(&self) -> TerminalStyle {
        let mut style = self.style;
        if self.invert {
            style.fg_color = Some(self.style.bg_color.unwrap_or(self.colors.black));
            style.bg_color = Some(self.style.fg_color.unwrap_or(self.colors.white));
        }
        style
    }

    fn linefeed(&mut self) {
        self.cursor.x = 0;
        self.cursor.y += 1;        
    }

    fn carriage_return(&mut self) {
        self.cursor.x = 0;        
    }

    fn horizontal_tab(&mut self) {
        self.cursor.x += 8 - (self.cursor.x % 8);
    }

    fn backspace(&mut self) {
        self.write_atom(self.cursor, None);
        self.cursor.x -= 1;
        if self.cursor.x < 0 {
            self.cursor.y -= 0;
            self.cursor.x = self.term_width - 1;
        }        
    }

    fn cursor_up(&mut self, n: usize) {
    }

    fn cursor_down(&mut self, n: usize) {        
    }

    fn save_cursor_position(&mut self) {
        self.cursor_save = self.cursor;
    }

    fn restore_cursor_position(&mut self) {
        self.cursor = self.cursor_save;
    }
}

impl Perform for PerfAtom {
    fn print(&mut self, c: char) {
        self.write_atom(self.cursor, Some(TerminalAtom::new(c, self.get_style())));

        self.cursor.x += 1;
        if self.cursor.x >= self.term_width {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.linefeed(),
            b'\r' => self.carriage_return(),
            b'\t' => self.horizontal_tab(),
            0x8 => self.backspace(),
            _ => {
                eprintln!("unhandled execute byte {:02x}", byte);
            }
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        /*
        eprintln!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
*/
    }

    fn put(&mut self, byte: u8) {
        //eprintln!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        //eprintln!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        //eprintln!("[osc_dispatch] params={:?} bell_terminated={}", params, bell_terminated);
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        let mut piter = params.into_iter();
        match c {
            // Set SGR
            'm' =>  while let Some(n) = piter.next() {
                match n[0] {
                    0 => {
                        self.style = TerminalStyle::default();
                        self.invert = false;
                    }
                    1 => self.style = self.style.add(TerminalStyle::bold(true)),
                    3 => self.style = self.style.add(TerminalStyle::italic(true)),
                    4 => self.style = self.style.add(TerminalStyle::underline(true)),
                    7 => self.invert = true,
                    27 => self.invert = false,

                    30 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.black)),
                    40 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.black)),
                    31 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.red)),
                    41 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.red)),
                    32 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.green)),
                    42 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.green)),
                    33 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.yellow)),
                    43 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.yellow)),
                    34 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.blue)),
                    44 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.blue)),
                    35 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.magenta)),
                    45 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.magenta)),
                    36 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.cyan)),
                    46 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.cyan)),
                    37 => self.style = self.style.add(TerminalStyle::fg_color(self.colors.white)),
                    47 => self.style = self.style.add(TerminalStyle::bg_color(self.colors.white)),

                    38 => {
                        let x = piter.next().unwrap();
                        match x[0] {
                            2 => {
                                let r = piter.next().unwrap();
                                let g = piter.next().unwrap();
                                let b = piter.next().unwrap();
                                self.style = self.style.add(TerminalStyle::fg_color((r[0] as u8, g[0] as u8, b[0] as u8)))
                            },
                            5 => {
                                let v = piter.next().unwrap();
                                self.style = self.style.add(TerminalStyle::fg_color(ansi_colours::rgb_from_ansi256(v[0] as u8)))
                            },
                            _ => {}
                        }
                    },
                    48 => {
                        let x = piter.next().unwrap();
                        match x[0] {
                            2 => {
                                let r = piter.next().unwrap();
                                let g = piter.next().unwrap();
                                let b = piter.next().unwrap();
                                self.style = self.style.add(TerminalStyle::bg_color((r[0] as u8, g[0] as u8, b[0] as u8)))
                            },
                            5 => {
                                let v = piter.next().unwrap();
                                self.style = self.style.add(TerminalStyle::bg_color(ansi_colours::rgb_from_ansi256(v[0] as u8)))
                            },
                            _ => {}
                        }
                    },

                    _ => {}
                }
            }
            '@' => {
                for x in self.cursor.x .. self.term_width {
                    self.write_atom(Point2::new(x, self.cursor.y), Some(TerminalAtom::new(' ', self.style)));
                }
            }
            'A' => {
                self.cursor.y -= piter.next().unwrap_or(&[1])[0] as i16;
            }
            'B' => {
                self.cursor.y += piter.next().unwrap_or(&[1])[0] as i16;
                if self.cursor.x >= self.term_width {
                    self.cursor.x = 0;
                }
            }
            'C' | 'a' => {
                self.cursor.x += piter.next().unwrap_or(&[1])[0] as i16;
                if self.cursor.x >= self.term_width {
                    self.cursor.y += self.cursor.x / self.term_width;
                    self.cursor.x %= self.term_width;
                }
            }
            'D' => {
                self.cursor.x -= piter.next().unwrap_or(&[1])[0] as i16;
                if self.cursor.x < 0 {
                    self.cursor.x = self.term_width - 1;
                    self.cursor.y -= 1;
                }
            }
            'd' => {
                self.cursor.y = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'E' => {
                if self.cursor.x >= self.term_width {
                    self.cursor.y += 1;
                }
                self.cursor.x = 0;
                self.cursor.y += piter.next().unwrap_or(&[1])[0] as i16;
            }
            'F' => {
                self.cursor.x = 0;
                self.cursor.y -= piter.next().unwrap_or(&[1])[0] as i16;
            }
            'G' | '`' => {
                self.cursor.x = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'H' | 'f' => {
                self.cursor.y = piter.next().unwrap_or(&[1])[0] as i16 - 1;
                self.cursor.x = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'J' => {
                let x = piter.next().unwrap_or(&[0 as u16; 1]);
                match x[0] {

                    // clear from cursor until end of screen
                    0 => {
                        let mut pos = self.cursor;

                        while pos.y < 100 {
                            self.write_atom(pos, None);
                            pos.x += 1;

                            if pos.x >= self.term_width {
                                pos.x = 0;
                                pos.y += 1;
                            }
                        }
                    },

                    // clear from cursor to begin
                    1 => {
                        let mut pos = self.cursor;
                        while pos.y >= 0 || pos.x >= 0 {
                            self.write_atom(pos, None);

                            pos.x -= 1;
                            if pos.x < 0 {
                                pos.x = self.term_width;
                                pos.y -= 1;
                            }                            
                        }

                        //self.cursor.x = 0;
                    }

                    // erase entire screen
                    2 => {
                        for y in 0 .. 100 {
                            for x in 0 .. self.term_width {
                                self.write_atom(Point2::new(x, y), None);
                            }
                        }

                        self.cursor = Point2::new(0, 0);
                    }

                    // invalid
                    _ => {}
                }
            }            
            'K' => {
                let x = piter.next().unwrap_or(&[0]);
                match x[0] {

                    // clear from cursor until end of line
                    0 => {
                        for x in self.cursor.x .. self.term_width {
                            self.write_atom(Point2::new(x, self.cursor.y), Some(TerminalAtom::new(' ', self.get_style())));
                        }
                    },

                    // clear from start of line until cursor
                    1 => {
                        for x in 0 .. self.cursor.x {
                            self.write_atom(Point2::new(x, self.cursor.y), Some(TerminalAtom::new(' ', self.get_style())));
                        }
                    },

                    // clear entire line
                    2 => {
                        for x in 0 .. self.term_width {
                            self.write_atom(Point2::new(x, self.cursor.y), Some(TerminalAtom::new(' ', self.get_style())));
                        }
                    },

                    // invalid
                    _ => {}
                }
            }
            /*
            'M' => {
                let n = piter.next().unwrap_or(&[1])[0] as i16;
                for y in 0 .. n {
                    for x in 0 .. self.term_width {
                        self.write_atom(Point2::new(x, self.cursor.y+y), None);
                    }
                }
            }
            'P' => {
                for x in 0 .. piter.next().unwrap_or(&[1])[0] {
                    self.backspace();
                }
            }
            'X' => {
                for x in 0 .. piter.next().unwrap_or(&[1])[0] {
                    self.write_atom(Point2::new(self.cursor.x + x as i16, self.cursor.y), None);
                }
            }
*/
            's' => {
                self.save_cursor_position();
            }
            'u' => {
                self.restore_cursor_position();
            }
            
            _ => {
                /*
                eprintln!(
                    "[csi_dispatch] params={:#?}, intermediates={:?}, ignore={:?}, char={:?}",
                    params, intermediates, ignore, c
                );
*/                
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {

        match (byte, intermediates) {
            //(b'B', intermediates) => configure_charset!(StandardCharset::Ascii, intermediates),
            (b'D', []) => self.linefeed(),
            (b'E', []) => {
                self.linefeed();
                self.carriage_return();
            },
            /*
            (b'H', []) => self.handler.set_horizontal_tabstop(),
            (b'M', []) => self.handler.reverse_index(),
            (b'Z', []) => self.handler.identify_terminal(None),
            (b'c', []) => self.handler.reset_state(),
            (b'0', intermediates) => {
                configure_charset!(StandardCharset::SpecialCharacterAndLineDrawing, intermediates)
            },
*/
            (b'7', []) => self.save_cursor_position(),
            //(b'8', [b'#']) => self.handler.decaln(),
            (b'8', []) => self.restore_cursor_position(),
/*
            (b'=', []) => self.handler.set_keypad_application_mode(),
            (b'>', []) => self.handler.unset_keypad_application_mode(),
**/
            // String terminator, do nothing (parser handles as string terminator).
            (b'\\', []) => (),
            _ => {
                /*
                eprintln!(
                    "unhandled esc_dispatch intermediates={:?}, ignore={:?}, byte={:02x}",
                    intermediates, ignore, byte
                );
*/
            }
        }
    }
}


