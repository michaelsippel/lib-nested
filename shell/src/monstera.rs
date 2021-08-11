
use {
    cgmath::Point2,
    nested::{
        core::{ViewPort, OuterViewPort},
        vec::VecBuffer,
        terminal::{
            TerminalStyle, TerminalView, make_label
        },
    }
};

pub fn make_monstera() -> OuterViewPort<dyn TerminalView> {
    let monstera_lines_port = ViewPort::new();
    let monstera_lines = VecBuffer::with_data(
        vec![
            make_label("                   |"),
            make_label("                   |"),
            make_label("             _..._ | _..._"),
            make_label("           .(     \\|/     )."),
            make_label("          (        |        )"),
            make_label("       .__>.   <>  |  <>   .<__."),
            make_label("      /            |            \\"),
            make_label("     | .___     _  |  _     ___. |"),
            make_label("     _./___>.  / \\ | / \\  .<___\\._ "),
            make_label("    /          \\_/ | \\_/          \\"),
            make_label("   (   .____.      |      .____.   )"),
            make_label("    ( /____  \\  _  |  _  /  ____\\ )"),
            make_label("    _*     \\.) / \\ | / \\ (./     *_"),
            make_label("   /           \\_/ | \\_/           \\"),
            make_label("   (    .__.       |       .__.    )"),
            make_label("    (  / __ \\      |      / __ \\  )"),
            make_label("      * /  \\.)  O  |  O  (./  \\ *"),
            make_label("       /   .___.   |   .___.   \\"),
            make_label("       (  / .---\\  |  /---. \\  )"),
            make_label("        *. (       |       ) .*"),
            make_label("             \\_ .  |   . _/"),
            make_label("                 \\ | /"),
            make_label("                   .")
        ],
        monstera_lines_port.inner()
    );

    monstera_lines_port.outer()
        .to_sequence()
        .to_index()
        .map_key(
            |idx| Point2::new(0 as i16, *idx as i16),
            |pt| if pt.x == 0 { Some(pt.y as usize) } else { None }
        )
        .flatten()
        .map_item(
            |p, at| at.add_style_back(TerminalStyle::fg_color((0,100,10)))
        )
}

