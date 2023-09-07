use {
    crate::{type_system::{TypeID}},
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
    pub fn unit() -> Self {
        TypeTerm::Ladder(vec![])
    }

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

    /// transform term to have at max 2 entries in Application list
    pub fn curry(&self) -> Self {
        match self {
            TypeTerm::App(head) => {
                let mut head = head.clone();
                if head.len() > 2 {
                    let mut tail = head.split_off(2);

                    TypeTerm::App(vec![
                        TypeTerm::App(head),
                        if tail.len() > 1 {
                            TypeTerm::App(tail).curry()
                        } else {
                            tail.remove(0)
                        }
                    ])
                } else {
                    TypeTerm::App(head)
                }
            }

            TypeTerm::Ladder(l) => {
                TypeTerm::Ladder( l.iter().map(|x| x.curry()).collect() )
            }

            atom => { atom.clone() }
        }
    }

    /// summarize all curried applications into one vec
    pub fn decurry(&mut self) -> &mut Self {
        match self {
            TypeTerm::App(args) => {
                if args.len() > 0 {
                    let mut a0 = args.remove(0);
                    a0.decurry();
                    match a0 {
                        TypeTerm::App(sub_args) => {
                            for (i,x) in sub_args.into_iter().enumerate() {
                                args.insert(i, x);
                            }
                        }
                        other => { args.insert(0, other); }
                    }
                }
            }
            TypeTerm::Ladder(args) => {
                for x in args.iter_mut() {
                    x.decurry();
                }
            }
            _ => {}
        }

        self
    }

    /// does the type contain ladders (false) or is it 'flat' (true) ?
    pub fn is_flat(&self) -> bool {
        match self {
            TypeTerm::TypeID(_) => true,
            TypeTerm::Num(_) => true,
            TypeTerm::Char(_) => true,
            TypeTerm::App(args) => args.iter().fold(true, |s,x| s && x.is_flat()),
            TypeTerm::Ladder(_) => false
        }
    }

    pub fn get_lnf_vec(self) -> Vec<TypeTerm> {
        match self.normalize() {
            TypeTerm::Ladder( v ) => {
                v
            },
            _ => {
                unreachable!();
            }
        }
    }

    /// transmute self into Ladder-Normal-Form
    ///
    /// Example:
    ///   <Seq <Digit 10>~Char> ⇒ <Seq <Digit 10>>~<Seq Char>
    pub fn normalize(self) -> Self {
        let mut new_ladder = Vec::<TypeTerm>::new();
        
        match self {
            TypeTerm::Ladder(args) => {
                for x in args.into_iter() {
                    new_ladder.push(x.normalize());
                }
            }

            TypeTerm::App(args) => {
                let mut args_iter = args.into_iter();
                if let Some(head) = args_iter.next() {

                    let mut stage1_args = vec![ head.clone() ];
                    let mut stage2_args = vec![ head.clone() ];

                    let mut done = false;

                    for x in args_iter {
                        match x.normalize() {
                            TypeTerm::Ladder(mut ladder) => {
                                // normalize this ladder

                                if !done {
                                    if ladder.len() > 2 {
                                        stage1_args.push( ladder.remove(0) );
                                        stage2_args.push( TypeTerm::Ladder(ladder.to_vec()) );
                                        done = true;
                                    } else if ladder.len() == 1 {
                                        stage1_args.push( ladder[0].clone() );
                                        stage2_args.push( ladder[0].clone() );
                                    } else {
                                        // empty type ?
                                    }

                                } else {
                                    stage1_args.push( TypeTerm::Ladder(ladder.clone()) );
                                    stage2_args.push( TypeTerm::Ladder(ladder.clone()) );
                                }
                            },
                            _ => {
                                unreachable!("x is in LNF");
                            }
                        }
                    }

                    new_ladder.push(TypeTerm::Ladder(stage1_args));
                    new_ladder.push(TypeTerm::Ladder(stage2_args));
                }
            }

            atom => {
                new_ladder.push(atom);
            }
        }

        TypeTerm::Ladder( new_ladder )
    }

    pub fn is_semantic_supertype_of(&self, t: &TypeTerm) -> Option<( usize, TypeTerm )> {
        t.is_semantic_subtype_of(self)
    }
    
    // returns ladder-step of first match and provided representation-type
    pub fn is_semantic_subtype_of(&self, expected_type: &TypeTerm) -> Option<( usize, TypeTerm )> {
        let provided_lnf = self.clone().get_lnf_vec();
        let expected_lnf = expected_type.clone().get_lnf_vec();

        for i in 0..provided_lnf.len() {
            if provided_lnf[i] == expected_lnf[0] {
                return Some((i, TypeTerm::Ladder(
                    provided_lnf[i..].into_iter().cloned().collect()
                )))
            }
        }

        None
    }

    pub fn is_syntactic_subtype_of(&self, expected_type: &TypeTerm) -> Result<usize, Option<(usize, usize)>> {
        if let Some((first_match, provided_type)) = self.is_semantic_subtype_of( expected_type ) {
            let provided_lnf = self.clone().get_lnf_vec();
            let expected_lnf = expected_type.clone().get_lnf_vec();

            let l = usize::min( provided_lnf.len(), expected_lnf.len() );
            
            for i in 0..l {

                if provided_lnf[i] != expected_lnf[i] {
                    return Err(Some((first_match, i)))
                }
            }

            Ok(l-1)
        } else {
            Err(None)
        }
    }

    /* this function is deprecated and only partially working,
    wontfix, will be replaced by TypeTerm-Editor
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
                    if let Some(f) = term_stack.last_mut() {
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
        }

        None
    }

    pub fn to_str(&self, names: &HashMap<TypeID, String>) -> String {
        match self {
            TypeTerm::App(args) => {
                let mut out = String::new();

                out.push_str(&"<");

                let mut first = true;
                for x in args.iter() {
                    if !first {
                        out.push_str(&" ");
                    } else {
                        first = false;
                    }

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
                    } else {
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
