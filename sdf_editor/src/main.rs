use{
    std::sync::{Arc, RwLock},
    cgmath::{Point2, Vector2},
    termion::event::{Event, Key},
    nested::{
        core::{
            View,
            ViewPort,
            Observer,
            OuterViewPort,
            port::UpdateTask
        },
        singleton::{SingletonBuffer, SingletonView},
        sequence::{SequenceView},
        integer::{PosIntEditor},
        terminal::{Terminal, TerminalCompositor, TerminalEvent, TerminalEditor},
        list::{ListEditor},
        tree_nav::{TreeNav}
    }
};


// projects a Sequence of ints to a color tuple
struct ColorCollector {
    src_view: Option<Arc<dyn SequenceView<Item = u32>>>,
    color: SingletonBuffer<(u8, u8, u8)>
}

impl ColorCollector {
    fn update(&mut self) {
        if let Some(l) = self.src_view.as_ref() {
            let r = l.get(&0).unwrap_or(0);
            let g = l.get(&1).unwrap_or(0);
            let b = l.get(&2).unwrap_or(0);

            self.color.set((r as u8, g as u8, b as u8));
        }
    }
}

impl Observer<dyn SequenceView<Item = u32>> for ColorCollector {
    fn reset(&mut self, new_view: Option<Arc<dyn SequenceView<Item = u32>>>) {
        self.src_view = new_view;
        self.update();
    }

    fn notify(&mut self, idx: &usize) {
        self.update();
    }
}


#[async_std::main]
async fn main() {
    let term_port = ViewPort::new();
    let compositor = TerminalCompositor::new(term_port.inner());

    let mut term = Terminal::new(term_port.outer());
    let term_writer = term.get_writer();

    let mut color_editor = ListEditor::new(
        || {
            Arc::new(RwLock::new(PosIntEditor::new(16)))
        },
        nested::list::ListEditorStyle::Clist
    );

    color_editor.goto(nested::tree_nav::TreeCursor {
        leaf_mode: nested::list::ListCursorMode::Insert,
        tree_addr: vec![ 0 ]
    });

    let color_port = ViewPort::new();
    let color_collector = Arc::new(RwLock::new(ColorCollector {
        src_view: None,
        color: SingletonBuffer::new((0, 0, 0), color_port.inner())
    }));

    color_editor.get_data_port().map(
        |sub_editor| sub_editor.read().unwrap().get_value()
    ).add_observer(
        color_collector
    );

    compositor.write().unwrap().push(color_editor.get_term_view().offset(Vector2::new(2, 2)));
    
    async_std::task::spawn(
        async move {
            loop {
                term_port.update();
                match term.next_event().await {
                    TerminalEvent::Input(Event::Key(Key::Ctrl('c'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('g'))) |
                    TerminalEvent::Input(Event::Key(Key::Ctrl('d'))) => break,

                    TerminalEvent::Input(Event::Key(Key::Left)) => {
                        color_editor.pxev();
                    }
                    TerminalEvent::Input(Event::Key(Key::Right)) => {
                        color_editor.nexd();
                    }
                    TerminalEvent::Input(Event::Key(Key::Up)) => {
                        color_editor.up();
                    }
                    TerminalEvent::Input(Event::Key(Key::Down)) => {
                        color_editor.dn();
                    }
                    TerminalEvent::Input(Event::Key(Key::Home)) => {
                        color_editor.goto_home();
                    }
                    TerminalEvent::Input(Event::Key(Key::End)) => {
                        color_editor.goto_end();
                    }
                    event => {
                        color_editor.handle_terminal_event(&event);
                    }
                }
            }
        }
    );
    
    term_writer.show().await.expect("output error!");
}

