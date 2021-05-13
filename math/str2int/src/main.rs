
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::io::FromRawFd
};

fn main() {
    nested::magic_header();
    eprintln!("       Parse MachineInt from String");
    nested::magic_header();

    eprintln!("
$1: radix
  ( ℕ )
  ( PositionalInt 10 BigEndian )
  ( Sequence ( Digit 10 ) )
  ( Sequence UTF-8-Char )
  ( Sequence MachineSyllab )
");

    eprintln!("
>0: n
  ( ℕ )
  ( PositionalInt $radix BigEndian )
  ( Sequence ( Digit $radix ) )
  ( Sequence UTF-8-Char )
  ( Stream UTF-8-Char )
  ( Stream MachineSyllab )
");

    eprintln!("
<1: n
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Stream MachineSyllab )
");

    nested::magic_header();

    let mut f0 = unsafe { File::from_raw_fd(0) };
    let mut f1 = unsafe { File::from_raw_fd(1) };

    let mut args = std::env::args();
    args.next().expect("Arg $0 missing!");

    let radix_str = args.next().expect("Arg $1 required!");

    let radix = u32::from_str_radix(&radix_str, 10).expect("could not parse radix");
    if radix > 16 {
        panic!("invalid radix! (radix<=16 required)");
    }

    let mut chars = Vec::new();
    f0.read_to_end(&mut chars).expect("");
    chars.retain(|c| (*c as char).is_alphanumeric());
    f1.write(&u64::from_str_radix(&String::from_utf8_lossy(&chars), radix).unwrap().to_le_bytes()).expect("");
}

