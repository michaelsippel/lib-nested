
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::io::FromRawFd
};

fn main() {
    nested::magic_header();
    eprintln!("       Parse MachineInt from String");
    nested::magic_header();

    let mut f0 = unsafe { File::from_raw_fd(0) };
    eprintln!("
> 0:
  ( ℕ )
  ( Sequence (Digit 10) )
  ( Sequence ASCII )
  ( Sequence MachineSlab )
  ( Pipe Shot (Sequence MachineSlab) )
");

    let mut f1 = unsafe { File::from_raw_fd(1) };
    eprintln!("
< 1:
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Array 8 MachineSlab )
  ( Pipe Shot (Array 8 MachineSlab) )
");

    nested::magic_header();

    let mut chars = Vec::new();
    f0.read_to_end(&mut chars);
    chars.retain(|c| (*c as char).is_numeric());
    f1.write(&u64::from_str_radix(&String::from_utf8_lossy(&chars), 10).unwrap().to_le_bytes());
}

