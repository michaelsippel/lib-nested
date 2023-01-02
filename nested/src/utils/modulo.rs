
pub fn modulo(a: isize, b: isize) -> isize {
    if b > 0 {
        ((a % b) + b) % b
    } else {
        0
    }
}


