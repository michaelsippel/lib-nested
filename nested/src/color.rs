use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth {
        0 => TerminalStyle::default(),
        1 => TerminalStyle::bg_color((20,20,20)),
        2 => TerminalStyle::default(),
        3 => TerminalStyle::default(),
        4 => TerminalStyle::default(),
        5 => TerminalStyle::default(),
        _ => TerminalStyle::bg_color((80,80,80))
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth % 3 {
        0 => TerminalStyle::fg_color((200, 200, 80)),
        1 => TerminalStyle::fg_color((80, 200, 200)).add(TerminalStyle::bold(true)),
        2 => TerminalStyle::fg_color((80, 80, 200)),
        3 => TerminalStyle::fg_color((200, 80, 200)),
        _ => TerminalStyle::default()
    }
}

