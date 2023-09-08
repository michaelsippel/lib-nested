use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth {
        0 => TerminalStyle::bg_color((150,80,230)),
        1 => TerminalStyle::bg_color((66,66,66)),
        2 => TerminalStyle::bg_color((44,44,44)),
        3 => TerminalStyle::bg_color((33,33,33)),
        4 => TerminalStyle::bg_color((28,28,28)),
        5 => TerminalStyle::bg_color((21,21,21)),
        _ => TerminalStyle::default(),
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    if depth  == 0 {
        TerminalStyle::fg_color((200, 200, 200))
    } else {
        match depth % 5 {
            0 => TerminalStyle::fg_color((128, 106, 97)),
            1 => TerminalStyle::fg_color((100, 120, 232)),
            2 => TerminalStyle::fg_color((180, 100, 96)),
            3 => TerminalStyle::fg_color((188, 155, 18)),
            4 => TerminalStyle::fg_color((135, 182, 134)),
            _ => TerminalStyle::default()
        }
    }
}

