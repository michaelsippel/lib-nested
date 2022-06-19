use {
    crate::terminal::TerminalStyle,
};

pub fn bg_style_from_depth(depth: usize) -> TerminalStyle {
    if depth == 0 {
        TerminalStyle::default()
    } else {
        TerminalStyle::bg_color((
            (30.0 / ( 0.90*depth as f64 )) as u8,
            (30.0 / ( 0.93*depth as f64 )) as u8,
            (50.0 / ( 0.95*depth as f64 )) as u8
        ))
    }
}

pub fn fg_style_from_depth(depth: usize) -> TerminalStyle {
    match depth % 3 {
        0 => TerminalStyle::fg_color((200, 200, 80)),
        1 => TerminalStyle::fg_color((80, 200, 200)),
        2 => TerminalStyle::fg_color((150, 150, 200)),
        _ => TerminalStyle::default()
    }
}

