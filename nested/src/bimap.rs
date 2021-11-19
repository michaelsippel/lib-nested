use std::{collections::HashMap, hash::Hash};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub struct Bimap<V: Eq + Hash, Λ: Eq + Hash> {
    pub mλ: HashMap<V, Λ>,
    pub my: HashMap<Λ, V>,
}

impl<V: Eq + Hash + Clone, Λ: Eq + Hash + Clone> Bimap<V, Λ> {
    pub fn new() -> Self {
        Bimap {
            mλ: HashMap::new(),
            my: HashMap::new(),
        }
    }

    pub fn insert(&mut self, y: V, λ: Λ) {
        self.mλ.insert(y.clone(), λ.clone());
        self.my.insert(λ, y);
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>
