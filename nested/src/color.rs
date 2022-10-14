use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth {
        1 => TerminalStyle::bg_color((40,40,40)),
        _ => TerminalStyle::default(),
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth % 6 {
        0 => TerminalStyle::fg_color((40, 180, 230)),
        1 => TerminalStyle::fg_color((120, 120, 120)),
        2 => TerminalStyle::fg_color((230, 180, 40)),
        3 => TerminalStyle::fg_color((80, 180, 200)),
        4 => TerminalStyle::fg_color((70, 90, 180)),
        5 => TerminalStyle::fg_color((200, 190, 70)),
        _ => TerminalStyle::default()
    }
}

