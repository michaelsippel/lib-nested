
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
> 0:
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Array 8 MachineSlab )
  ( Pipe Shot (Array 8 MachineSlab) )
");

    eprintln!("
< 1:
  ( ℕ )
  ( Sequence (Digit 10) )
  ( Sequence ASCII )
  ( Sequence MachineSlab )
  ( Pipe Shot (Sequence MachineSlab) )
");

    nested::magic_header();

    let mut bytes = [0 as u8; 8];
    f0.read_exact(&mut bytes);
    println!("{}", u64::from_le_bytes(bytes));
}

