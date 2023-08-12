use {
    crate::{
        type_system::{TypeID}
    },
    std::collections::HashMap
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeTerm {
    /* Atomic Terms */

    // Base types from dictionary
    TypeID(TypeID),

    // Literals
    Num(i64),
    Char(char),


    /* Complex Terms */

    // Type Parameters
    // avoid currying to save space & indirection
    App(Vec< TypeTerm >),

    // Type Ladders
    Ladder(Vec< TypeTerm >),
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TypeTerm {
    pub fn new(id: TypeID) -> Self {
        TypeTerm::TypeID(id)
    }

    pub fn arg(&mut self, t: impl Into<TypeTerm>) -> &mut Self {
        match self {
            TypeTerm::App(args) => {
                args.push(t.into());                
            }

            _ => {
                *self = TypeTerm::App(vec![
                    self.clone(),
                    t.into()
                ])                
            }
        }

        self
    }

    pub fn num_arg(&mut self, v: i64) -> &mut Self {
        self.arg(TypeTerm::Num(v))
    }

    pub fn char_arg(&mut self, c: char) -> &mut Self {
        self.arg(TypeTerm::Char(c))
    }

    /* TODO

    // summarize all curried applications into one vec
    pub fn decurry() {}

    // transmute into Ladder-Normal-Form
    pub fn normalize(&mut self) {
        match self {
            TypeTerm::Apply{ id, args } => {
                
            }
        }
    }

    pub fn is_supertype_of(&self, t: &TypeTerm) -> bool {
        t.is_subtype_of(self)
    }

    pub fn is_subtype_of(&self, t: &TypeTerm) -> bool {
        false
    }
    */

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
                            let mut chars = atom.chars();
                            let first = chars.next().unwrap();

                            if first.is_numeric() {
                                f.num_arg(i64::from_str_radix(atom, 10).unwrap());
                            } else if first == '\'' {
                                if let Some(mut c) = chars.next() {
                                    if c == '\\' {
                                        if let Some('n') = chars.next() {
                                            c = '\n';
                                        }
                                    }
                                    f.char_arg(c);
                                }
                            } else {
                                f.arg(TypeTerm::new(
                                    names.get(atom)
                                    .expect(&format!("invalid atom {}", atom)).clone()
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

    pub fn to_str(&self, names: &HashMap<TypeID, String>) -> String {
        match self {
            TypeTerm::App(args) => {
                let mut out = String::new();

                out.push_str(&"<");

                for x in args.iter() {
                    out.push_str(&" ");
                    out.push_str(&x.to_str(names));
                }

                out.push_str(&">");

                out
            }

            TypeTerm::Ladder(l) => {
                let mut out = String::new();

                let mut first = true;
                for x in l.iter() {
                    if !first {
                        out.push_str(&"~");
                        first = false;
                    }
                    out.push_str(&x.to_str(names));
                }

                out                
            }

            TypeTerm::Num(n) => format!("{}", n),
            TypeTerm::Char('\n') => format!("'\\n'"),
            TypeTerm::Char(c) => format!("'{}'", c),
            TypeTerm::TypeID(id) => format!("{}", names[id]),
        }
    }
}
