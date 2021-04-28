
use {
    std::sync::{Arc, RwLock},
    nested::{
        core::{
            View,
            ViewPort,
            Observer,
            ObserverBroadcast,
            InnerViewPort,
            OuterViewPort
        },
        sequence::{SequenceView, VecBuffer},
        integer::{RadixProjection}
    }
};

#[async_std::main]
async fn main() {
    nested::magic_header();
    eprintln!("    Convert Radix of Positional Integer");
    nested::magic_header();

    let mut args = std::env::args();
    args.next().expect("Arg $0 missing!");

    eprintln!("
$1: src_radix
  ( ℕ )
  ( PositionalInt 10 LittleEndian )
  ( Sequence (Digit 10) )
  ( Sequence Ascii )
  ( ArgString )
");
    let src_radix_str = args.next().expect("Arg $1 required!");

    
    eprintln!("
$2: dst_radix
  ( ℕ )
  ( PositionalInt 10 LittleEndian )
  ( Sequence (Digit 10) )
  ( Sequence Ascii )
  ( ArgString )
");
    let dst_radix_str = args.next().expect("Arg $2 required!");
    
    eprintln!("
>0: n
  ( ℕ ) ~~ <1
  ( PositionalInt src_radix LittleEndian )
  ( Sequence (Digit src_radix) )
  ( Sequence MachineInt )
  ( PipeStream bincode (SequenceDiff MachineInt) )
");

    eprintln!("
<1: n
  ( ℕ ) ~~ >0
  ( PositionalInt dst_radix LittleEndian )
  ( Sequence (Digit dst_radix) )
  ( Sequence MachineInt )
  ( PipeStream bincode (SequenceDiff MachineInt) )
");

    nested::magic_header();

    let src_radix = usize::from_str_radix(&src_radix_str, 10).expect("could not parse src_radix");
    let dst_radix = usize::from_str_radix(&dst_radix_str, 10).expect("could not parse dst_radix");

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

