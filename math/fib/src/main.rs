
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::io::FromRawFd
};

fn fib(n: u64) -> u64 {
    let mut y = 0;
    let mut y1 = 1;
    let mut y2 = 0;

    for _ in 0 .. n {
        y = y1 + y2;
        y2 = y1;
        y1 = y;
    }

    y
}

fn main() {
    nested::magic_header();

    eprintln!("            Fibonacci Sequence");

    nested::magic_header();

    eprintln!("
interface (Sequence ℕ) 0 1");

    let mut f0 = unsafe { File::from_raw_fd(0) };
    eprintln!("
> 0: n
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Pipe Shot MachineWord )
");

    let mut f1 = unsafe { File::from_raw_fd(1) };
    eprintln!("
< 1: n'th fibonacci number
  ( ℕ )
  ( MachineInt )
  ( MachineWord )
  ( Pipe Shot MachineWord )
");

    nested::magic_header();

    let mut bytes = [0 as u8; 8];
    f0.read_exact(&mut bytes);
    let n = u64::from_le_bytes(bytes);
    bytes = fib(n).to_le_bytes();
    f1.write(&bytes);
}

