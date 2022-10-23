use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth {
        0 => TerminalStyle::bg_color((150,80,230)),
        1 => TerminalStyle::bg_color((35,35,35)),
        2 => TerminalStyle::bg_color((20,20,20)),
        _ => TerminalStyle::default(),
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth % 6 {
        0 => TerminalStyle::fg_color((40, 180, 230)),
        1 => TerminalStyle::fg_color((120, 120, 120)),
        2 => TerminalStyle::fg_color((250, 165, 40)),
        3 => TerminalStyle::fg_color((80, 180, 200)),
        4 => TerminalStyle::fg_color((180, 240, 85)),
        5 => TerminalStyle::fg_color((200, 190, 70)),
        _ => TerminalStyle::default()
    }
}

