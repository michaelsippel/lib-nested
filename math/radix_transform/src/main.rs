
use {
    nested::{
        core::{
            ViewPort,
            TypeDict
        },
        sequence::{VecBuffer},
        integer::{RadixProjection}
    }
};

#[async_std::main]
async fn main() {
    let mut td = TypeDict::new();
    for tn in vec![
        "MachineWord", "MachineInt", "MachineSyllab",
        "Vec", "Stream", "Json",
        "Sequence", "UTF-8-Char",
        "PositionalInt", "Digit", "LittleEndian", "BigEndian",
        "DiffStream", "ℕ",
        "$src_radix", "$dst_radix"
    ] { td.add_typename(tn.into()); }

    let radix_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt 10 LittleEndian )").unwrap(),
        td.type_term_from_str("( Sequence ( Digit 10 ) )").unwrap(),
        td.type_term_from_str("( Sequence UTF-8-Char )").unwrap(),
        td.type_term_from_str("( Sequence MachineSyllab )").unwrap()
    ];

    let src_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt $src_radix LittleEndian )").unwrap(),
        td.type_term_from_str("( Sequence ( Digit $src_radix ) )").unwrap(),
        td.type_term_from_str("( Sequence MachineInt )").unwrap(),
        td.type_term_from_str("( DiffStream ( Vec MachineInt ) )").unwrap(),
        td.type_term_from_str("( Json )").unwrap(),
        td.type_term_from_str("( Stream UTF-8-Char )").unwrap(),
        td.type_term_from_str("( Stream MachineSyllab )").unwrap()
    ];

    let dst_types = vec![
        td.type_term_from_str("( ℕ )").unwrap(),
        td.type_term_from_str("( PositionalInt $dst_radix LittleEndian )").unwrap(),
        td.type_term_from_str("( Sequence ( Digit $dst_radix ) )").unwrap(),
        td.type_term_from_str("( Sequence MachineInt )").unwrap(),
        td.type_term_from_str("( DiffStream ( Vec MachineInt ) )").unwrap(),
        td.type_term_from_str("( Json )").unwrap(),
        td.type_term_from_str("( Stream UTF-8-Char )").unwrap(),
        td.type_term_from_str("( Stream MachineSyllab )").unwrap()
    ];

    nested::magic_header();
    eprintln!("    Convert Radix of Positional Integer");
    nested::magic_header();

    eprintln!("\n$1: src_radix");
    for t in radix_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    eprintln!("\n$2: dst_radix");
    for t in radix_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    eprintln!("\n>0: n");
    for t in src_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    eprintln!("\n<1: n");
    for t in dst_types.iter() {
        eprintln!("  {}", td.type_term_to_str(t));
    }

    nested::magic_header();

    let mut args = std::env::args();
    args.next().expect("Arg $0 missing!");

    let src_radix_str = args.next().expect("Arg $1 required!");
    let dst_radix_str = args.next().expect("Arg $2 required!");

    let src_radix = usize::from_str_radix(&src_radix_str, 10).expect("could not parse src_radix");
    let dst_radix = usize::from_str_radix(&dst_radix_str, 10).expect("could not parse dst_radix");

    assert!(src_radix > 1);
    assert!(dst_radix > 1);

    let src_digits_port = ViewPort::new();
    let dst_digits_port = ViewPort::new();

    let mut src_digits = VecBuffer::<usize>::new(src_digits_port.inner());

    let _proj = RadixProjection::new(
        src_radix,
        dst_radix,
        src_digits_port.outer().to_sequence(),
        dst_digits_port.inner()
    );

    // output dst digits
    let writer = {
        use std::{
            os::unix::io::FromRawFd
        };

        dst_digits_port.outer().serialize_json(unsafe { std::fs::File::from_raw_fd(1) })
    };

    // start reading src digits
    {
        use async_std::{
            os::unix::io::FromRawFd
        };

        src_digits.from_json(unsafe { async_std::fs::File::from_raw_fd(0) }).await;
    }

    drop(writer);
}

