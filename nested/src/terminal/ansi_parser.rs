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
        core::{View, InnerViewPort, OuterViewPort, ViewPort, Observer, ObserverBroadcast},
        projection::ProjectionHelper,
        terminal::{
            TerminalAtom,
            TerminalStyle,
            TerminalView
        },
        singleton::{
            SingletonBuffer,
            SingletonView
        },
        index::{
            buffer::IndexBuffer,
            IndexView,
            IndexArea
        }
    },
    cgmath::{Vector2, Point2},
    vte::{Params, Parser, Perform}
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub fn read_ansi_from<R: Read + Unpin>(ansi_reader: &mut R, port: InnerViewPort<dyn TerminalView>) {
    let mut statemachine = Parser::new();

    let buf_port = ViewPort::new();
    let size_port = ViewPort::<dyn SingletonView<Item = Vector2<i16>>>::new();
    let cursor_port = ViewPort::<dyn SingletonView<Item = Point2<i16>>>::new();
    let offset_port = ViewPort::<dyn SingletonView<Item = Vector2<i16>>>::new();

    let mut performer = PerfAtom {
        buf: IndexBuffer::new(buf_port.inner()),
        size: SingletonBuffer::new(Vector2::new(120, 40), size_port.inner()),
        offset: SingletonBuffer::new(Vector2::new(0, 0), offset_port.inner()),
        cursor: SingletonBuffer::new(Point2::new(0, 0), cursor_port.inner()),
        cursty: TerminalStyle::default(),
        curinv: false,
        cursav: Point2::new(0, 0),

        colors: ColorPalett {
            black: (1, 1, 1),
            red: (222, 56, 43),
            green: (0, 64, 0),
            yellow: (255, 199, 6),
            blue: (0, 111, 184),
            magenta: (118, 38, 113),
            cyan: (44, 181, 233),
            white: (204, 204, 204)
        },

        pty_proj: PtyView::new(
            buf_port.outer(),
            cursor_port.outer().map(|x| Some(x)),
            offset_port.outer().map(|x| Some(x)),
            size_port.outer().map(|x| Some(x)),

            port
        )
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

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

enum TTYColor {
    Rgb(u8, u8, u8),

    // 3-Bit/4-Bit colors
    Black,  LightBlack,
    Red,    LightRed,
    Green,  LightGreen,
    Yellow, LightYellow,
    Blue,   LightBlue,
    Magenta,LightMagenta,
    Cyan,   LightCyan,
    White,  LightWhite,
}

struct ColorPalett {
    black: (u8, u8, u8),
    red: (u8, u8, u8),
    green: (u8, u8, u8),
    yellow: (u8, u8, u8),
    blue: (u8, u8, u8),
    magenta: (u8, u8, u8),
    cyan: (u8, u8, u8),
    white: (u8, u8, u8),
}

impl ColorPalett {
    fn get_rgb(&self, col: &TTYColor) -> (u8, u8, u8) {
        match col {
            TTYColor::Rgb(r,g,b) => (*r,*g,*b),
            TTYColor::Black | TTYColor::LightBlack => self.black,
            TTYColor::Red | TTYColor::LightRed => self.red,
            TTYColor::Green | TTYColor::LightGreen => self.green,
            TTYColor::Yellow | TTYColor::LightYellow => self.yellow,
            TTYColor::Blue | TTYColor::LightBlue => self.blue,
            TTYColor::Magenta | TTYColor::LightMagenta => self.magenta,
            TTYColor::Cyan | TTYColor::LightCyan => self.cyan,
            TTYColor::White | TTYColor::LightWhite => self.white,
        }
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

/// Display Cursor & scrolling
struct PtyView {
    buf: Arc<dyn IndexView<Point2<i16>, Item = TerminalAtom>>,
    curpos: Arc<dyn SingletonView<Item = Option<Point2<i16>>>>,
    offset: Arc<dyn SingletonView<Item = Option<Vector2<i16>>>>,
    size: Arc<dyn SingletonView<Item = Option<Vector2<i16>>>>,

    old_offset: Vector2<i16>,
    old_curpos: Point2<i16>,
    old_size: Vector2<i16>,

    max_pt: Point2<i16>,

    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<usize, Self>
}

impl View for PtyView {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for PtyView {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        let atom = self.buf.get(&(pt + self.old_offset));
        if self.old_curpos == *pt {
            if let Some(mut a) = atom {
                let bg_col = a.style.fg_color.unwrap_or((255,255,255));
                let fg_col = a.style.bg_color.unwrap_or((0,0,0));
                a.style.fg_color = Some(fg_col);
                a.style.bg_color = Some(bg_col);
                Some(a)
            } else {
                Some(TerminalAtom::new(' ', TerminalStyle::bg_color((255, 255, 255))))
            }
        } else {
            atom
        }
    }

    fn area(&self) -> IndexArea<Point2<i16>> {        
        IndexArea::Range(Point2::new(0, 0) ..= Point2::new(
            std::cmp::max(self.old_curpos.x, self.max_pt.x),
            std::cmp::max(self.old_curpos.y, self.max_pt.y)
        ))
    }
}

impl PtyView {
    fn new(
        buf_port: OuterViewPort<dyn IndexView<Point2<i16>, Item = TerminalAtom>>,
        curpos_port: OuterViewPort<dyn SingletonView<Item = Option<Point2<i16>>>>,
        offset_port: OuterViewPort<dyn SingletonView<Item = Option<Vector2<i16>>>>,
        size_port: OuterViewPort<dyn SingletonView<Item = Option<Vector2<i16>>>>,

        out_port: InnerViewPort<dyn TerminalView>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(
                PtyView {
                    old_curpos: Point2::new(0, 0),
                    old_size: Vector2::new(0, 0),
                    old_offset: Vector2::new(0, 0),
                    max_pt: Point2::new(0, 0),

                    curpos: proj_helper.new_singleton_arg(
                        0,
                        curpos_port,
                        |s: &mut Self, _msg| {
                            s.cast.notify(&IndexArea::Set(vec![ s.old_curpos ]));
                            s.old_curpos = s.curpos.get().unwrap_or(Point2::new(0,0));
                            s.cast.notify(&IndexArea::Set(vec![ s.old_curpos ]));
                        }),

                    offset: proj_helper.new_singleton_arg(
                        1,
                        offset_port,
                        |s: &mut Self, _msg| {
                            // todo
                            let new_offset = s.offset.get().unwrap_or(Vector2::new(0, 0));
                            if s.old_offset != new_offset {
                                s.old_offset = new_offset;
                                s.cast.notify(&s.area());
                            }
                        }),

                    size: proj_helper.new_singleton_arg(
                        2,
                        size_port,
                        |s: &mut Self, _msg| {
                            let new_size = s.size.get().unwrap_or(Vector2::new(0, 0));
                            if s.old_size != new_size {
                                s.old_size = new_size;
                                s.cast.notify(&s.area());
                            }
                        }),

                    buf: proj_helper.new_index_arg(
                        3,
                        buf_port,
                        |s: &mut Self, area| {
                            let size = s.old_size;
                            let area = area.map(
                                |pt| {
                                    *pt - s.old_offset
                                }
                            );

                            if s.max_pt.x < size.x || s.max_pt.y < size.y {
                                match &area {
                                    IndexArea::Empty => {}
                                    IndexArea::Full => {}
                                    IndexArea::Range(_) => {}
                                    IndexArea::Set(v) => {
                                        let mx = v.iter().map(|pt| pt.x).max().unwrap_or(0);
                                        if mx > s.max_pt.x && mx < size.x {
                                            s.max_pt.x = mx;
                                        }

                                        let my = v.iter().map(|pt| pt.y).max().unwrap_or(0);
                                        if my > s.max_pt.y && my < size.y {
                                            s.max_pt.y = my;
                                        }
                                    }
                                }
                            }

                            s.cast.notify(&area);
                        }),

                    cast: out_port.get_broadcast(),
                    proj_helper
                }
            ));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.set_view(Some(proj.clone()));

        proj        
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

struct PerfAtom {
    buf: IndexBuffer<Point2<i16>, TerminalAtom>,
    size: SingletonBuffer<Vector2<i16>>,
    offset: SingletonBuffer<Vector2<i16>>,
    cursor: SingletonBuffer<Point2<i16>>,
    cursty: TerminalStyle,
    curinv: bool,
    cursav: Point2<i16>,

    colors: ColorPalett,

    pty_proj: Arc<RwLock<PtyView>>
}

impl PerfAtom {
    fn write_atom(&mut self, pos: Point2<i16>, atom: Option<TerminalAtom>) {
        if let Some(mut a) = atom {
            self.buf.insert(pos + self.offset.get(), a);
        } else {
            self.buf.remove(pos);
        }
    }

    fn get_style(&self) -> TerminalStyle {
        let mut style = self.cursty;
        if self.curinv {
            style.fg_color = Some(self.cursty.bg_color.unwrap_or(self.colors.black));
            style.bg_color = Some(self.cursty.fg_color.unwrap_or(self.colors.white));
        }
        style
    }

    fn set_fg_color(&mut self, col: &TTYColor) {
        self.cursty = self.cursty.add(TerminalStyle::fg_color(self.colors.get_rgb(col)));
    }

    fn set_bg_color(&mut self, col: &TTYColor) {
        self.cursty = self.cursty.add(TerminalStyle::bg_color(self.colors.get_rgb(col)));
    }

    fn linefeed(&mut self) {
        let size = self.size.get();
        let mut c = self.cursor.get_mut();
        c.x = 0;

        if c.y+1 >= size.y {
            self.scroll_up(1);
        } else {
            c.y += 1;
        }
    }

    fn carriage_return(&mut self) {
        let mut c = self.cursor.get_mut();
        c.x = 0;        
    }

    fn horizontal_tab(&mut self) {
        let mut c = self.cursor.get_mut();
        c.x += 8 - (c.x % 8);
    }

    fn backspace(&mut self) {
        //self.write_atom(self.cursor.get(), None);
        let mut c = self.cursor.get_mut();
        c.x -= 1;
        if c.x < 0 {
            c.y -= 0;
            c.x = self.size.get().x - 1;
        }
    }

    fn cursor_up(&mut self, n: usize) {
        self.cursor.get_mut().y -= n as i16;
    }

    fn cursor_dn(&mut self, n: usize) {
        self.cursor.get_mut().y += n as i16;

        // todo: scroll ?
    }

    fn scroll_up(&mut self, n: usize) {
        self.offset.get_mut().y += n as i16;
    }

    fn scroll_dn(&mut self, n: usize) {
        self.offset.get_mut().y -= n as i16;
    }

    fn save_cursor_position(&mut self) {
        self.cursav = self.cursor.get();
    }

    fn restore_cursor_position(&mut self) {
        self.cursor.set(self.cursav);
    }
}

impl Perform for PerfAtom {
    fn print(&mut self, ch: char) {
        let mut c = self.cursor.get_mut();
        self.write_atom(*c, Some(TerminalAtom::new(ch, self.get_style())));

        c.x += 1;
        if c.x >= self.size.get().x {
            self.linefeed();
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.linefeed(),
            b'\r' => self.carriage_return(),
            b'\t' => self.horizontal_tab(),
            0x8 => self.backspace(),
            _ => {
                //eprintln!("unhandled execute byte {:02x}", byte);
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
                        self.cursty = TerminalStyle::default();
                        self.curinv = false;
                    }
                    1 => self.cursty = self.cursty.add(TerminalStyle::bold(true)),
                    3 => self.cursty = self.cursty.add(TerminalStyle::italic(true)),
                    4 => self.cursty = self.cursty.add(TerminalStyle::underline(true)),
                    7 => self.curinv = true,
                    27 => self.curinv = false,

                    30 => self.set_fg_color(&TTYColor::Black),
                    40 => self.set_bg_color(&TTYColor::Black),
                    90 => self.set_fg_color(&TTYColor::LightBlack),
                    100 => self.set_bg_color(&TTYColor::LightBlack),
                    31 => self.set_fg_color(&TTYColor::Red),
                    41 => self.set_bg_color(&TTYColor::Red),
                    91 => self.set_fg_color(&TTYColor::LightRed),
                    101 => self.set_bg_color(&TTYColor::LightRed),
                    32 => self.set_fg_color(&TTYColor::Green),
                    42 => self.set_bg_color(&TTYColor::Green),
                    92 => self.set_fg_color(&TTYColor::LightGreen),
                    102 => self.set_bg_color(&TTYColor::LightGreen),
                    33 => self.set_fg_color(&TTYColor::Yellow),
                    43 => self.set_bg_color(&TTYColor::Yellow),
                    93 => self.set_fg_color(&TTYColor::LightYellow),
                    103 => self.set_bg_color(&TTYColor::LightYellow),
                    34 => self.set_fg_color(&TTYColor::Blue),
                    44 => self.set_bg_color(&TTYColor::Blue),
                    94 => self.set_fg_color(&TTYColor::LightBlue),
                    104 => self.set_bg_color(&TTYColor::LightBlue),
                    35 => self.set_fg_color(&TTYColor::Magenta),
                    45 => self.set_bg_color(&TTYColor::Magenta),
                    95 => self.set_fg_color(&TTYColor::LightMagenta),
                    105 => self.set_bg_color(&TTYColor::LightMagenta),
                    36 => self.set_fg_color(&TTYColor::Cyan),
                    46 => self.set_bg_color(&TTYColor::Cyan),
                    96 => self.set_fg_color(&TTYColor::LightCyan),
                    106 => self.set_bg_color(&TTYColor::LightCyan),
                    37 => self.set_fg_color(&TTYColor::White),
                    47 => self.set_bg_color(&TTYColor::White),
                    97 => self.set_fg_color(&TTYColor::LightWhite),
                    107 => self.set_bg_color(&TTYColor::LightWhite),

                    38 => {
                        let x = piter.next().unwrap();
                        match x[0] {
                            2 => {
                                let r = piter.next().unwrap()[0] as u8;
                                let g = piter.next().unwrap()[0] as u8;
                                let b = piter.next().unwrap()[0] as u8;
                                self.set_fg_color(&TTYColor::Rgb(r,g,b));
                            },
                            5 => {
                                let v = piter.next().unwrap();
                                let rgb = ansi_colours::rgb_from_ansi256(v[0] as u8);
                                self.set_fg_color(&TTYColor::Rgb(rgb.0, rgb.1, rgb.2));
                            },
                            _ => {}
                        }
                    },
                    48 => {
                        let x = piter.next().unwrap();
                        match x[0] {
                            2 => {
                                let r = piter.next().unwrap()[0] as u8;
                                let g = piter.next().unwrap()[0] as u8;
                                let b = piter.next().unwrap()[0] as u8;
                                self.set_bg_color(&TTYColor::Rgb(r,g,b));
                            },
                            5 => {
                                let v = piter.next().unwrap();
                                let rgb = ansi_colours::rgb_from_ansi256(v[0] as u8);
                                self.set_bg_color(&TTYColor::Rgb(rgb.0, rgb.1, rgb.2));
                            },
                            _ => {}
                        }
                    },

                    _ => {}
                }
            }
            '@' => {
                let c = self.cursor.get();
                for x in c.x .. self.size.get().x {
                    self.write_atom(Point2::new(x, c.y), Some(TerminalAtom::new(' ', self.cursty)));
                }
            }
            'A' => { self.cursor_up(piter.next().unwrap_or(&[1])[0] as usize); }
            'B' => { self.cursor_dn(piter.next().unwrap_or(&[1])[0] as usize); }
            'C' | 'a' => {
                let mut c = self.cursor.get_mut();
                c.x += piter.next().unwrap_or(&[1])[0] as i16;
                if c.x >= self.size.get().x {
                    c.y += c.x / self.size.get().x;
                    c.x %= self.size.get().x;
                }
            }
            'D' => {
                let mut c = self.cursor.get_mut();
                c.x -= piter.next().unwrap_or(&[1])[0] as i16;
                if c.x < 0 {
                    c.x = self.size.get().x - 1;
                    c.y -= 1;
                }
            }
            'd' => {
                self.cursor.get_mut().y = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'E' => {
                let mut c = self.cursor.get_mut();
                if c.x >= self.size.get().x {
                    c.y += 1;
                }
                c.x = 0;
                c.y += piter.next().unwrap_or(&[1])[0] as i16;
            }
            'F' => {
                let mut c = self.cursor.get_mut();
                c.x = 0;
                c.y -= piter.next().unwrap_or(&[1])[0] as i16;
            }
            'G' | '`' => {
                self.cursor.get_mut().x = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'H' | 'f' => {
                let mut c = self.cursor.get_mut();
                c.y = piter.next().unwrap_or(&[1])[0] as i16 - 1;
                c.x = piter.next().unwrap_or(&[1])[0] as i16 - 1;
            }
            'J' => {
                let x = piter.next().unwrap_or(&[0 as u16; 1]);
                match x[0] {

                    // clear from cursor until end of screen
                    0 => {
                        let mut pos = self.cursor.get();

                        while pos.y < 100 {
                            self.write_atom(pos, None);
                            pos.x += 1;

                            if pos.x >= self.size.get().x {
                                pos.x = 0;
                                pos.y += 1;
                            }
                        }
                    },

                    // clear from cursor to begin
                    1 => {
                        let mut pos = self.cursor.get();
                        while pos.y >= 0 || pos.x >= 0 {
                            self.write_atom(pos, None);

                            pos.x -= 1;
                            if pos.x < 0 {
                                pos.x = self.size.get().x;
                                pos.y -= 1;
                            }                            
                        }

                        //self.cursor.x = 0;
                    }

                    // erase entire screen
                    2 => {
                        for y in 0 .. 100 {
                            for x in 0 .. self.size.get().x {
                                self.write_atom(Point2::new(x, y), None);
                            }
                        }

                        self.cursor.set(Point2::new(0, 0));
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
                        let c = self.cursor.get();
                        for x in c.x .. self.size.get().x {
                            self.write_atom(Point2::new(x, c.y), Some(TerminalAtom::new(' ', self.get_style())));
                        }
                    },

                    // clear from start of line until cursor
                    1 => {
                        let c = self.cursor.get();
                        for x in 0 .. c.x {
                            self.write_atom(Point2::new(x, c.y), Some(TerminalAtom::new(' ', self.get_style())));
                        }
                    },

                    // clear entire line
                    2 => {
                        let c = self.cursor.get();
                        for x in 0 .. self.size.get().x {
                            self.write_atom(Point2::new(x, c.y), Some(TerminalAtom::new(' ', self.get_style())));
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
                    for x in 0 .. self.size.get().x {
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
            'S' => { self.scroll_up(piter.next().unwrap_or(&[1])[0] as usize); }
            'T' => { self.scroll_dn(piter.next().unwrap_or(&[1])[0] as usize); }
            's' => { self.save_cursor_position(); }
            'u' => { self.restore_cursor_position(); }            
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


