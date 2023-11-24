
impl TreeNav {
    fn get_cursor_widget(&self) -> OuterViewPort<dyn TerminalView> {
        VecBuffer::with_data(
            vec![
                make_label("@").with_fg_color((150, 80,230)),
                self.get_addr_view()
                    .map(|i|
                        make_label(&format!("{}", i)).with_fg_color((0, 100, 20)))
                    .separate(make_label(".").with_fg_color((150, 80,230)))
                    .to_grid_horizontal()
                    .flatten(),
                make_label(":").with_fg_color((150, 80,230)),
                self.get_mode_view()
                    .map(|mode| {
                        make_label(
                            match mode {
                                ListCursorMode::Insert => "INSERT",
                                ListCursorMode::Select => "SELECT"
                            })
                            .with_fg_color((200, 200, 20))
                    })
                    .to_grid()
                    .flatten(),
                make_label(":").with_fg_color((150, 80,230))
            ]
        ).get_port()
            .to_sequence()
            .to_grid_horizontal()
            .flatten()
    }
}

