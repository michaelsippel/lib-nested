use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth {
        0 => TerminalStyle::bg_color((150,80,230)),
        1 => TerminalStyle::bg_color((75,75,75)),
        2 => TerminalStyle::bg_color((40,40,40)),
        3 => TerminalStyle::bg_color((30,30,30)),
        4 => TerminalStyle::bg_color((25,25,25)),
        5 => TerminalStyle::bg_color((20,20,20)),
        _ => TerminalStyle::default(),
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth % 6 {
        0 => TerminalStyle::fg_color((120, 120, 0)),
        1 => TerminalStyle::fg_color((250, 165, 40)),
        2 => TerminalStyle::fg_color((80, 180, 180)),
        3 => TerminalStyle::fg_color((180, 240, 85)),
        4 => TerminalStyle::fg_color((200, 190, 70)),
        _ => TerminalStyle::default()
    }    
}

