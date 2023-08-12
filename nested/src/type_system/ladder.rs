use {
    crate::type_system::{TypeTerm, TypeID},
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeLadder(pub Vec<TypeTerm>);

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl From<Vec<TypeTerm>> for TypeLadder {
    fn from(l: Vec<TypeTerm>) -> Self {
        TypeLadder(l)
    }
}

impl From<TypeTerm> for TypeLadder {
    fn from(l: TypeTerm) -> Self {
        TypeLadder(vec![ l ])
    }
}

impl TypeLadder {
    /// if compatible, returns the number of descents neccesary
    pub fn is_compatible_with(&self, other: &TypeLadder) -> Option<usize> {
        if let Some(other_top_type) = other.0.first() {
            for (i, t) in self.0.iter().enumerate() {
                if t == other_top_type {
                    return Some(i);
                }
            }

            None
        } else {
            None
        }
    }

    pub fn is_matching_repr(&self, other: &TypeLadder) -> Result<usize, Option<(usize, usize)>> {
        if let Some(start) = self.is_compatible_with(other) {
            for (i, (t1, t2)) in self.0.iter().skip(start).zip(other.0.iter()).enumerate() {
                if t1 != t2 {
                    return Err(Some((start, i)));
                }
            }

            Ok(start)
        } else {
            Err(None)
        }
    }

    pub fn to_str1(&self, names: &HashMap<TypeID, String>) -> String {
        let mut s = String::new();
        let mut first = true;

        for t in self.0.iter() {
            if !first {
                s = s + "~";
            }
            first = false;
            s = s + &t.to_str(names);
        }
        s
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

