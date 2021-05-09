use {
    std::{
        sync::Arc,
        any::Any,
        ops::Deref,
        collections::HashMap,
        iter::Peekable
    },
    crate::{
        bimap::Bimap,
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub type TypeID = u64;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TypeTerm {
    Type {
        id: TypeID,
        args: Vec<TypeTerm>
    },
    Num(i64)
}

impl TypeTerm {
    pub fn new(id: TypeID) -> Self {
        TypeTerm::Type{ id, args: vec![] }
    }

    pub fn arg(&mut self, t: TypeTerm) -> &mut Self {
        if let TypeTerm::Type{ id, args } = self {
            args.push(t);
        }

        self
    }

    pub fn num_arg(&mut self, v: i64) -> &mut Self {
        self.arg(TypeTerm::Num(v))
    }

    pub fn from_str(s: &str, names: &HashMap<String, u64>) -> Option<Self> {
        let mut term_stack = Vec::<Option<TypeTerm>>::new();

        for token in s.split_whitespace() {
            match token {
                "(" => {
                    term_stack.push(None);
                },
                ")" => {
                    let t = term_stack.pop().unwrap();
                    if term_stack.len() > 0 {
                        let mut f = term_stack.last_mut().unwrap();
                        if let Some(f) = f {
                            f.arg(t.unwrap());
                        } else {
                            //error
                        }
                    } else {
                        return t;
                    }
                },
                atom => {
                    let mut f = term_stack.last_mut().unwrap();

                    match f {
                        Some(f) =>
                            if atom.chars().nth(0).unwrap().is_numeric() {
                                f.num_arg(i64::from_str_radix(atom, 10).unwrap());
                            } else {
                                f.arg(TypeTerm::new(*names.get(atom).expect(&format!("invalid atom {}", atom))));
                            }
                        None => {
                            *f = Some(TypeTerm::new(*names.get(atom).expect(&format!("invalid atom {}", atom))));
                        }
                    }
                }
            }
        }

        None
    }

    // only adds parenthesis where args.len > 0
    pub fn to_str1(&self, names: &HashMap<u64, String>) -> String {
        match self {
            TypeTerm::Type{ id, args } =>
                if args.len() > 0 {
                    format!(
                        "( {} {})",
                        names[id],
                        if args.len() > 0 {
                            args.iter().fold(
                                String::new(),
                                |str, term| format!("{}{} ", str, term.to_str1(names) )
                            )
                        } else {
                            String::new()
                        }
                    )
                } else {
                    names[id].clone()
                },

            TypeTerm::Num(n) =>
                format!("{}", n)
        }
    }

    // always adds an enclosing pair of parenthesis
    pub fn to_str(&self, names: &HashMap<u64, String>) -> String {
        match self {
            TypeTerm::Type{ id, args } =>
                format!(
                    "( {} {})",
                    names[id],
                    if args.len() > 0 {
                        args.iter().fold(
                            String::new(),
                            |str, term| format!("{}{} ", str, term.to_str1(names) )
                        )
                    } else {
                        String::new()
                    }),

            TypeTerm::Num(n) =>
                format!("{}", n)
        }
    }

}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TypeDict {
    typenames: Bimap::<String, u64>,
    type_id_counter: u64
}

impl TypeDict {
    pub fn new() -> Self {
        TypeDict {
            typenames: Bimap::new(),
            type_id_counter: 0
        }
    }

    pub fn add_typename(&mut self, tn: String) {
        self.typenames.insert(tn, self.type_id_counter);
        self.type_id_counter += 1;
    }

    pub fn type_term_from_str(&self, typename: &str) -> Option<TypeTerm> {
        TypeTerm::from_str(typename, &self.typenames.mλ)
    }

    pub fn type_term_to_str(&self, term: &TypeTerm) -> String {
        term.to_str(&self.typenames.my)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

