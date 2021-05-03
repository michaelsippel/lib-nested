
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::io::FromRawFd
};

fn main() {
    nested::magic_header();
    eprintln!("     Human-readably Print MachineInt");
    nested::magic_header();

    let mut f0 = unsafe { File::from_raw_fd(0) };
    eprintln!("
>0:
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Stream MachineSyllab )
");

    eprintln!("
<1:
  ( ℕ )
  ( PositionalInt 10 BigEndian )
  ( Sequence ( Digit 10 ) )
  ( Sequence UTF-8-Char )
  ( Stream UTF-8-Char )
  ( Stream MachineSyllab )
");

    nested::magic_header();

    let mut bytes = [0 as u8; 8];
    f0.read_exact(&mut bytes);
    println!("{}", u64::from_le_bytes(bytes));
}

