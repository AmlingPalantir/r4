use std::collections::HashMap;

pub struct NameTrie<T> {
    t: Option<T>,
    children: HashMap<char, NameTrie<T>>,
}

impl<T> Default for NameTrie<T> {
    fn default() -> Self {
        return NameTrie {
            t: None,
            children: HashMap::new(),
        };
    }
}

impl<T> NameTrie<T> {
    pub fn get(&self, name: &str) -> Vec<&T> {
        let mut n = self;
        for c in name.chars() {
            match n.children.get(&c) {
                None => {
                    return vec![];
                }
                Some(ref n2) => {
                    n = n2;
                }
            }
        }
        if let Some(ref t) = n.t {
            // Exact, honor even if there are longer matches.
            return vec![t]
        }
        loop {
            let mut acc = Vec::<&T>::new();
            n.collect(&mut acc);
            return acc;
        }
    }

    fn collect<'a, 'b: 'a>(&'b self, acc: &'a mut Vec<&'b T>) {
        if let Some(ref t) = self.t {
            acc.push(t);
        }
        for n in self.children.values() {
            n.collect(acc);
        }
    }

    pub fn insert(&mut self, name: &str, t: T) {
        let n = name.chars().fold(self, |n, c| n.children.entry(c).or_insert_with(NameTrie::default));
        if n.t.is_some() {
            panic!("Collision in options at {}", name);
        }
        n.t = Some(t);
    }
}
