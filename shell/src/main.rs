
extern crate portable_pty;

mod monstera;
mod process;
mod pty;
mod ascii_box;

use{
    std::sync::{Arc, RwLock},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            InnerViewPort,
            OuterViewPort,
            Observer,
            ObserverExt,
            ObserverBroadcast,
            context::{ReprTree, Object, MorphismType, MorphismMode, Context},
            port::{UpdateTask}},
        index::{IndexView, IndexArea},
        grid::{GridWindowIterator},
        sequence::{SequenceView, SequenceViewExt},
        vec::{VecBuffer},
        integer::{RadixProjection, DigitEditor, PosIntEditor},
        terminal::{
            Terminal,
            TerminalStyle,
            TerminalAtom,
            TerminalCompositor,
            TerminalEvent,
            make_label,
            TerminalView,
            TerminalEditor},
        string_editor::{StringEditor},
        tree_nav::{TreeNav, TreeNavResult, TreeCursor, TerminalTreeEditor},
        list::{SExprView, ListCursorMode, ListEditor, ListEditorStyle},
        projection::ProjectionHelper
    },
    crate::{
        process::ProcessLauncher
    }
};

struct TestView {}

impl View for TestView {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for TestView {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        Some(TerminalAtom::from('.'))
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Full
    }
}

struct Plot {
    limit: usize,
    data: Arc<dyn SequenceView<Item = usize>>,
    cast: Arc<RwLock<ObserverBroadcast<dyn TerminalView>>>,
    proj_helper: ProjectionHelper<(), Self>
}

impl View for Plot {
    type Msg = IndexArea<Point2<i16>>;
}

impl IndexView<Point2<i16>> for Plot {
    type Item = TerminalAtom;

    fn get(&self, pt: &Point2<i16>) -> Option<TerminalAtom> {
        if pt.y >= 0 {
            if let Some(cur_val) = self.data.get(&(pt.x as usize)) {
                if cur_val <= self.limit {
                    if pt.y == (self.limit - cur_val) as i16 {
                        return Some(TerminalAtom::from('*'));
                    }
                }
                if pt.x > 0 {
                    if let Some(prev_val) = self.data.get(&((pt.x-1) as usize)) {
                        if
                            (
                                pt.y > (self.limit - prev_val) as i16 &&
                                pt.y < (self.limit - cur_val) as i16
                            )
                            ||
                            (
                                pt.y < (self.limit - prev_val) as i16 &&
                                pt.y > (self.limit - cur_val) as i16
                            )
                        {
                            return Some(TerminalAtom::from('|'));
                        }
                    }
                }
            }
        }
        None
    }

    fn area(&self) -> IndexArea<Point2<i16>> {
        IndexArea::Range(
            Point2::new(0,0)
                ..= Point2::new(
                    self.data.len().unwrap_or(0) as i16,
                    self.limit as i16
                )
        )
    }
}

impl Plot {
    pub fn new(
        data_port: OuterViewPort<dyn SequenceView<Item = usize>>,
        out_port: InnerViewPort<dyn TerminalView>
    ) -> Arc<RwLock<Self>> {
        let mut proj_helper = ProjectionHelper::new(out_port.0.update_hooks.clone());
        let proj = Arc::new(RwLock::new(
            Plot {
                data: proj_helper.new_sequence_arg(
                    (),
                    data_port,
                    |s: &mut Self, idx| {
                        let val = s.data.get(idx).unwrap_or(0);

                        if val > s.limit {
                            s.limit = val;
                            s.cast.notify(&s.area());
                        } else {
                            s.cast.notify(&IndexArea::Range(
                                Point2::new(*idx as i16, 0)
                                    ..= Point2::new(*idx as i16, s.limit as i16)
                            ));
                        }
                    }
                ),

                limit: 0,
                cast: out_port.get_broadcast(),
                proj_helper
            }
        ));

        proj.write().unwrap().proj_helper.set_proj(&proj);
        out_port.set_view(Some(proj.clone()));

        proj
    }
}

#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    async_std::task::spawn(
        async move {
            let table_port = ViewPort::<dyn nested::grid::GridView<Item = OuterViewPort<dyn TerminalView>>>::new();
            let mut table_buf = nested::index::buffer::IndexBuffer::new(table_port.inner());

            let magic =
                make_label("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>")
                .map_item(
                    |pos, atom|
                    atom.add_style_back(
                        TerminalStyle::fg_color(
                            (5,
                             ((80+(pos.x*30)%100) as u8),
                             (55+(pos.x*15)%180) as u8)
                        )
                    )
                );

            let cur_size_port = ViewPort::new();
            let mut cur_size = nested::singleton::SingletonBuffer::new(Vector2::new(10, 10), cur_size_port.inner());

            let status_chars_port = ViewPort::new();
            let mut status_chars = VecBuffer::new(status_chars_port.inner());

            let mut process_list_editor = ListEditor::new(
                Box::new(|| {
                    Arc::new(RwLock::new(
                        ProcessLauncher::new()
                    ))
                }),
                ListEditorStyle::VerticalSexpr
            );

            
            let plist_vec_port = ViewPort::new();
            let mut plist = VecBuffer::new(plist_vec_port.inner());

            async_std::task::spawn(async move {
                let (w, h) = termion::terminal_size().unwrap();
                let mut x : usize = 0;
                loop {
                    let val =
                        (
                            5.0 + (x as f32 / 3.0).sin() * 5.0 +
                            2.0 + ((7+x) as f32 / 5.0).sin() * 2.0 +
                            2.0 + ((9+x) as f32 / 10.0).cos() * 3.0
                        ) as usize;

                    if x < w as usize {
                        plist.push(val);
                    } else {
                        *plist.get_mut(x % (w as usize)) = val;
                    }

                    x+=1;
                    async_std::task::sleep(std::time::Duration::from_millis(10)).await;

                    if x%(w as usize) == 0 {
                        async_std::task::sleep(std::time::Duration::from_secs(3)).await;
                    }
                }
            });

            let plot_port = ViewPort::new();
            let plot = Plot::new(plist_vec_port.outer().to_sequence(), plot_port.inner());

            table_buf.insert_iter(vec![
                (Point2::new(0, 0), magic.clone()),
                (Point2::new(0, 1), status_chars_port.outer().to_sequence().to_grid_horizontal()),
                (Point2::new(0, 2), magic.clone()),
                (Point2::new(0, 3), process_list_editor.get_term_view()),
            ]);

            let (w, h) = termion::terminal_size().unwrap();

            compositor.write().unwrap().push(
                plot_port.outer()
                    .map_item(|pt,a| {
                        a.add_style_back(TerminalStyle::fg_color((255 - pt.y as u8 * 8, 100, pt.y as u8 *15)))
                    })
                    .offset(Vector2::new(0,h as i16-20))
            );

            compositor.write().unwrap().push(
                monstera::make_monstera()
                    .offset(Vector2::new(w as i16-38, 0)));

            compositor.write().unwrap().push(
                table_port.outer()
                    .flatten()
                    .offset(Vector2::new(3, 0))
            );

            process_list_editor.goto(TreeCursor {
                leaf_mode: ListCursorMode::Insert,
                tree_addr: vec![ 0 ]
            });

            let tp = term_port.clone();
            async_std::task::spawn(
                async move {
                    loop {
                        tp.update();
                        async_std::task::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }
            );
            
            loop {
                let ev = term.next_event().await;
                match ev {
                    TerminalEvent::Resize(new_size) => {
                        cur_size.set(new_size);
                        term_port.inner().get_broadcast().notify(&IndexArea::Full);
                    }
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                    TerminalEvent::Input(Event::Key(Key::Left)) => {
                        process_list_editor.pxev();
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        process_list_editor.nexd();
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => {
                        if process_list_editor.up() == TreeNavResult::Exit {
                            process_list_editor.dn();
                            process_list_editor.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Down)) => {
                        if process_list_editor.dn() == TreeNavResult::Continue {
                            process_list_editor.goto_home();
                        }
                    }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        process_list_editor.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        process_list_editor.goto_end();
                    }
                    TerminalEvent::Input(Event::Key(Key::Char('\n'))) => {
                        if let Some(launcher) = process_list_editor.get_item() {
                            launcher.write().unwrap().launch_pty2();
                        }
                    }

                    ev => {
                        if process_list_editor.get_cursor().leaf_mode == ListCursorMode::Select {
                            match ev {
                                TerminalEvent::Input(Event::Key(Key::Char('l'))) => { process_list_editor.up(); },
                                TerminalEvent::Input(Event::Key(Key::Char('a'))) => { process_list_editor.dn(); },
                                TerminalEvent::Input(Event::Key(Key::Char('i'))) => { process_list_editor.pxev(); },
                                TerminalEvent::Input(Event::Key(Key::Char('e'))) => { process_list_editor.nexd(); },
                                TerminalEvent::Input(Event::Key(Key::Char('u'))) => { process_list_editor.goto_home(); },
                                TerminalEvent::Input(Event::Key(Key::Char('o'))) => { process_list_editor.goto_end(); },
                                _ => {
                                    process_list_editor.handle_terminal_event(&ev);
                                }
                            }
                        } else {
                            process_list_editor.handle_terminal_event(&ev);
                        }
                    }
                }

                status_chars.clear();
                let cur = process_list_editor.get_cursor();

                if cur.tree_addr.len() > 0 {
                    status_chars.push(TerminalAtom::new('@', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                    for x in cur.tree_addr {
                        for c in format!("{}", x).chars() {
                            status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((0, 100, 20))));
                        }
                        status_chars.push(TerminalAtom::new('.', TerminalStyle::fg_color((120, 80, 80))));
                    }

                    status_chars.push(TerminalAtom::new(':', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                    for c in
                        match cur.leaf_mode {
                            ListCursorMode::Insert => "INSERT",
                            ListCursorMode::Select => "SELECT",
                            ListCursorMode::Modify => "MODIFY"
                        }.chars()
                    {
                        status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((200, 200, 20))));
                    }
                    status_chars.push(TerminalAtom::new(':', TerminalStyle::fg_color((120, 80, 80)).add(TerminalStyle::bold(true))));
                } else {
                    for c in "Press <DN> to enter".chars() {
                        status_chars.push(TerminalAtom::new(c, TerminalStyle::fg_color((200, 200, 20))));
                    }
                }
            }

            drop(term);
            drop(term_port);
        }
    );

    term_writer.show().await.expect("output error!");
}

