use {
    crate::{
        utils::Bimap,
        type_system::{TypeTerm}
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum TypeID {
    Fun(u64),
    Var(u64)
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct TypeDict {
    typenames: Bimap<String, TypeID>,
    type_lit_counter: u64,
    type_var_counter: u64,
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

impl TypeDict {
    pub fn new() -> Self {
        TypeDict {
            typenames: Bimap::new(),
            type_lit_counter: 0,
            type_var_counter: 0,
        }
    }

    pub fn add_varname(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Var(self.type_var_counter);
        self.type_var_counter += 1;
        self.typenames.insert(tn, tyid.clone());
        tyid
    }

    pub fn add_typename(&mut self, tn: String) -> TypeID {
        let tyid = TypeID::Fun(self.type_lit_counter);
        self.type_lit_counter += 1;
        self.typenames.insert(tn, tyid.clone());
        tyid
    }

    pub fn get_typename(&self, tid: &TypeID) -> Option<String> {
        self.typenames.my.get(tid).cloned()
    }

    pub fn get_typeid(&self, tn: &String) -> Option<TypeID> {
        self.typenames.mλ.get(tn).cloned()
    }

    pub fn type_term_from_str(&self, typename: &str) -> Option<TypeTerm> {
        TypeTerm::from_str(typename, &self.typenames.mλ)
    }

    pub fn type_term_to_str(&self, term: &TypeTerm) -> String {
        term.to_str(&self.typenames.my)
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

