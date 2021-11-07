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
        term_width: 180,

        cursor_stack: Vec::new(),

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

    cursor_stack: Vec<Point2<i16>>,

    buf: IndexBuffer<Point2<i16>, TerminalAtom>,
}

impl PerfAtom {
    fn write_atom(&mut self, pos: Point2<i16>, atom: Option<TerminalAtom>) {
        if let Some(a) = atom {
            self.buf.insert(pos, a);
        } else {
            self.buf.remove(pos);
        }
    }
}

impl Perform for PerfAtom {
    fn print(&mut self, c: char) {
        //eprintln!("[print] {:?}", c);
        self.write_atom(
            self.cursor,
            Some(TerminalAtom::new(c, self.style))
        );

        self.cursor.x += 1;
        if self.cursor.x > self.term_width {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        //eprintln!("[execute] {:02x}", byte);
        match byte {
            // linefeed
            b'\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
            },

            // carriage return
            b'\r' => {
                self.cursor.x = 0;
            },

            // horizontal tab
            b'\t' => {
                self.cursor.x += 8 - (self.cursor.x % 8);
            },

            // backspace
            0x8 => {
                self.cursor.x -= 1;
                if self.cursor.x < 0 {
                    self.cursor.y -= 0;
                    self.cursor.x = self.term_width - 1;
                }
            }
            _ => {}
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
        /*
        eprintln!(
            "[csi_dispatch] params={:#?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
         */

        let mut piter = params.into_iter();

        match c {
            // Set SGR
            'm' =>  while let Some(n) = piter.next() {
                match n[0] {
                        0 => self.style = TerminalStyle::default(),
                        1 => self.style = self.style.add(TerminalStyle::bold(true)),
                        3 => self.style = self.style.add(TerminalStyle::italic(true)),
                        4 => self.style = self.style.add(TerminalStyle::underline(true)),

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
                                    self.style = self.style.add(TerminalStyle::fg_color((r[0] as u8, g[0] as u8, b[30] as u8)))
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
                                    self.style = self.style.add(TerminalStyle::bg_color((r[0] as u8, g[0] as u8, b[30] as u8)))
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
            },

            'H' => {
                if let Some(y) = piter.next() { self.cursor.y = y[0] as i16 - 1 };
                if let Some(x) = piter.next() { self.cursor.x = x[0] as i16 - 1 };

                //eprintln!("cursor at {:?}", self.cursor);
            },

            'A' => { self.cursor.y -= piter.next().unwrap()[0] as i16; }
            'B' => { self.cursor.y += piter.next().unwrap()[0] as i16; }
            'C' => { self.cursor.x += piter.next().unwrap()[0] as i16; }
            'D' => { self.cursor.x -= piter.next().unwrap()[0] as i16; }
            'E' => {
                self.cursor.x = 0;
                self.cursor.y += piter.next().unwrap()[0] as i16;
            }
            'F' => {
                self.cursor.x = 0;
                self.cursor.y -= piter.next().unwrap()[0] as i16;
            }
            'G' => {
                self.cursor.x = piter.next().unwrap()[0] as i16 - 1;
            }

            's' => {
                self.cursor_stack.push(self.cursor);
            }
            'u' => {
                if let Some(c) = self.cursor_stack.pop() {
                    self.cursor = c;
                }
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

                        //self.cursor.
                    }

                    // erase entire screen
                    2 => {
                        for y in 0 .. 100 {
                            for x in 0 .. self.term_width {
                                self.write_atom(Point2::new(x, y), None);
                            }
                        }
                    }

                    // invalid
                    _ => {}
                }
            }
            
            'K' => {
                let x = piter.next().unwrap();
                match x[0] {

                    // clear cursor until end
                    0 => {
                        for x in self.cursor.x .. self.term_width {
                            self.write_atom(Point2::new(x, self.cursor.y), None);
                        }
                    },

                    // clear start until cursor
                    1 => {
                        for x in 0 .. self.cursor.x {
                            self.write_atom(Point2::new(x, self.cursor.y), None);
                        }                        
                    },

                    // clear entire line
                    2 => {
                        for x in 0 .. self.term_width {
                            self.write_atom(Point2::new(x, self.cursor.y), None);
                        }                        
                    },

                    // invalid
                    _ => {}
                }
            }
            
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        /*
        eprintln!(
            "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
        intermediates, ignore, byte
        );
         */

        match byte {

            
            
            _ => {}
        }
    }
}


