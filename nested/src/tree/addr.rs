
pub struct TreeAddr(pub Vec<usize>);

impl From<Vec<usize>> for TreeAddr {
    fn from(v: Vec<usize>) -> TreeAddr {
        TreeAddr(v)
    }
}

