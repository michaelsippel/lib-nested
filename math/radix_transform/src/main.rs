
use {
    std::sync::{Arc, RwLock},
    nested::{
        core::{
            View,
            ViewPort,
            Observer,
            ObserverBroadcast,
            InnerViewPort,
            OuterViewPort,
            TypeTerm,
            TypeDict
        },
        sequence::{SequenceView, VecBuffer},
        integer::{RadixProjection}
    }
};

#[async_std::main]
async fn main() {
    let mut td = TypeDict::new();
    for tn in vec![
        "MachineWord", "MachineInt", "MachineSlab",
        "Vec", "NullTerminatedString",
        "Sequence", "Ascii",
        "PositionalInt", "Digit", "LittleEndian", "BigEndian",
        "DiffStream", "ℕ"
    ] { td.add_typename(tn.into()); }

    let radix_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt 10 LittleEndian )").unwrap(),
        td.type_term_from_str("( Sequence ( Digit 10 ) )").unwrap(),
        td.type_term_from_str("( Sequence Ascii )").unwrap(),
        td.type_term_from_str("( Sequence MachineSlab )").unwrap()
    ];

    nested::magic_header();
    eprintln!("    Convert Radix of Positional Integer");
    nested::magic_header();

    let mut args = std::env::args();
    args.next().expect("Arg $0 missing!");

    eprintln!("\n$1: src_radix");
    for t in radix_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    eprintln!("\n$2: dst_radix");
    for t in radix_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    let src_radix_str = args.next().expect("Arg $1 required!");
    let dst_radix_str = args.next().expect("Arg $2 required!");

    let src_radix = usize::from_str_radix(&src_radix_str, 10).expect("could not parse src_radix");
    let dst_radix = usize::from_str_radix(&dst_radix_str, 10).expect("could not parse dst_radix");

    let in_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt )").unwrap()
            .num_arg(src_radix as i64)
            .arg(td.type_term_from_str("( LittleEndian )").unwrap())
            .clone(),

        td.type_term_from_str("( Sequence )").unwrap()
            .arg(
                td.type_term_from_str("( Digit )").unwrap()
                    .num_arg(src_radix as i64).clone()
            )
            .clone(),

        td.type_term_from_str("( Sequence MachineInt )").unwrap(),
        td.type_term_from_str("( DiffStream ( Vec MachineInt ) )").unwrap(),
    ];

    let out_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt )").unwrap()
            .num_arg(dst_radix as i64)
            .arg(td.type_term_from_str("( LittleEndian )").unwrap()).clone(),

        td.type_term_from_str("( Sequence )").unwrap()
            .arg(
                td.type_term_from_str("( Digit )").unwrap()
                    .num_arg(dst_radix as i64).clone()
            )
            .clone(),

        td.type_term_from_str("( Sequence MachineInt )").unwrap(),
        td.type_term_from_str("( DiffStream ( Vec MachineInt ) )").unwrap(),
    ];

    eprintln!("\n>0: n");
    for t in in_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    eprintln!("\n<1: n");
    for t in out_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    nested::magic_header();

    let src_digits_port = ViewPort::new();
    let dst_digits_port = ViewPort::new();

    let mut src_digits = VecBuffer::<usize>::new(src_digits_port.inner());

    let proj = RadixProjection::new(
        src_radix,
        dst_radix,
        src_digits_port.outer().to_sequence(),
        dst_digits_port.inner()
    );

    // output dst digits
    let writer = {
        use std::{
            fs::File,
            io::{Read, Write},
            os::unix::io::FromRawFd
        };

        dst_digits_port.outer().serialize_json(unsafe { std::fs::File::from_raw_fd(1) })
    };

    // start reading src digits
    {
        use async_std::{
            fs::File,
            io::{Read, Write},
            os::unix::io::FromRawFd
        };

        src_digits.from_json(unsafe { async_std::fs::File::from_raw_fd(0) }).await;
    }
}

