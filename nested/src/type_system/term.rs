use {
    crate::{
        type_system::{TypeLadder, TypeID}
    },
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeTerm {
    Type {
        id: u64,
        args: Vec< TypeLadder >
    },
    Var(u64),
    Num(i64),
    Char(char)
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TypeTerm {
    pub fn new(id: TypeID) -> Self {
        match id {
            TypeID::Fun(id) => TypeTerm::Type { id, args: vec![] },
            TypeID::Var(_) => {unreachable!();}
        }
    }

    pub fn arg(&mut self, t: impl Into<TypeLadder>) -> &mut Self {
        if let TypeTerm::Type { id: _, args } = self {
            args.push(t.into());
        }

        self
    }

    pub fn num_arg(&mut self, v: i64) -> &mut Self {
        self.arg(TypeTerm::Num(v))
    }

    pub fn char_arg(&mut self, c: char) -> &mut Self {
        self.arg(TypeTerm::Char(c))
    }

    pub fn from_str(s: &str, names: &HashMap<String, TypeID>) -> Option<Self> {
        let mut term_stack = Vec::<Option<TypeTerm>>::new();

        for token in s.split_whitespace() {
            match token {
                "(" => {
                    term_stack.push(None);
                }
                ")" => {
                    let t = term_stack.pop().unwrap();
                    if term_stack.len() > 0 {
                        let f = term_stack.last_mut().unwrap();
                        if let Some(f) = f {
                            f.arg(t.unwrap());
                        } else {
                            //error
                        }
                    } else {
                        return t;
                    }
                }
                atom => {
                    let f = term_stack.last_mut().unwrap();

                    match f {
                        Some(f) => {
                            if atom.chars().nth(0).unwrap().is_numeric() {
                                f.num_arg(i64::from_str_radix(atom, 10).unwrap());
                            } else {
                                f.arg(TypeTerm::new(
                                    names.get(atom).expect(&format!("invalid atom {}", atom)).clone(),
                                ));
                            }
                        }
                        None => {
                            *f = Some(TypeTerm::new(
                                names.get(atom).expect(&format!("invalid atom {}", atom)).clone(),
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    // only adds parenthesis where args.len > 0
    pub fn to_str1(&self, names: &HashMap<TypeID, String>) -> String {
        match self {
            TypeTerm::Type { id, args } => {
                if args.len() > 0 {
                    format!(
                        "({}{})",
                        names[&TypeID::Fun(*id)],
                        if args.len() > 0 {
                            args.iter().fold(String::new(), |str, term| {
                                format!(" {}{}", str, term.to_str1(names))
                            })
                        } else {
                            String::new()
                        }
                    )
                } else {
                    names[&TypeID::Fun(*id)].clone()
                }
            }

            TypeTerm::Num(n) => format!("{}", n),
            TypeTerm::Char(c) => format!("'{}'", c),
            TypeTerm::Var(varid) => format!("T"),
        }
    }

    // always adds an enclosing pair of parenthesis
    pub fn to_str(&self, names: &HashMap<TypeID, String>) -> String {
        match self {
            TypeTerm::Type { id, args } => format!(
                "( {} {})",
                names[&TypeID::Fun(*id)],
                if args.len() > 0 {
                    args.iter().fold(String::new(), |str, term| {
                        format!("{}{} ", str, term.to_str1(names))
                    })
                } else {
                    String::new()
                }
            ),

            TypeTerm::Num(n) => format!("{}", n),
            TypeTerm::Char(c) => format!("'{}'", c),
            TypeTerm::Var(varid) => format!("T"),
        }
    }
}
