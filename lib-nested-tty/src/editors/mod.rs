
pub mod list;

use {
    nested::{
        edit_tree::{EditTree},
        repr_tree::{ReprTree, Context}
    },
    r3vi::{
        view::{singleton::*, sequence::*},
        projection::decorate_sequence::*
    },
    crate::{
        make_label,
        DisplaySegment,
        atom::{TerminalAtom, TerminalStyle}
    }
};


pub fn edittree_make_char_view(
    node: EditTree
) -> EditTree {
    eprintln!("nested-tty: EditTree make char-view");
    node.disp.view
        .write().unwrap()
        .insert_branch(ReprTree::from_view(
            Context::parse(&node.ctx, "TerminalView"),
            node.get_edit::< nested::editors::char::CharEditor >()
                .unwrap()
                .read()
                .unwrap()
                .get_port()
                .map(move |c| TerminalAtom::from(if c == '\0' { ' ' } else { c }))
                .to_grid()
        ));

    node
}

pub fn edittree_make_digit_view(
    node: EditTree
) -> EditTree {
    node.disp.view
        .write().unwrap()
        .insert_branch(ReprTree::from_view(
            Context::parse(&node.ctx, "TerminalView"),
            node.get_edit::< nested::editors::digit::DigitEditor >()
                .unwrap()
                .read()
                .unwrap()
                .get_data_port()
                .map(move |digit|
                    match digit {
                        Ok(digit) => TerminalAtom::new( char::from_digit(digit, 16).unwrap_or('?'), TerminalStyle::fg_color((220, 220, 0)) ),
                        Err(c) => TerminalAtom::new( c, TerminalStyle::fg_color((220, 0, 0)) )
                    }
                )
                .to_grid()
        ));

    node
}

/*
pub fn edittree_make_seq_view(
    mut node: EditTree
) -> EditTree {
    node.disp.view
        .write().unwrap()
        .insert_branch(ReprTree::new_leaf(
            Context::parse(&node.ctx, "TerminalView"),
            node.get_edit::< nested::editors::list::ListEditor >()
                .unwrap()
                .read().unwrap()
                .get_data_port()
                .map(move |item_edittree|
                    edittree_make_tty_view( item_edittree ).display_view()
                )
                .wrap(make_label("("), make_label(")"))
                .to_grid_horizontal()
                .flatten()
                .into()
        ));
    node
}
*/
