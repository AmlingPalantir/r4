use std::collections::HashMap;

pub struct NameTrie<T> {
    t: Option<T>,
    children: HashMap<char, NameTrie<T>>,
}

impl<T> NameTrie<T> {
    pub fn new() -> NameTrie<T> {
        return NameTrie {
            t: None,
            children: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> &T {
        let mut n = self;
        for c in name.chars() {
            match n.children.get(&c) {
                None => {
                    panic!("No option {}", name);
                }
                Some(ref n2) => {
                    n = n2;
                }
            }
        }
        if let Some(ref t) = n.t {
            // Exact, honor even if there are longer matches.
            return t;
        }
        loop {
            if let Some(ref t) = n.t {
                if n.children.is_empty() {
                    return t;
                }
                panic!("Ambiguous option {}", name);
            }

            let mut iter = n.children.iter();
            let first = iter.next().unwrap();
            if iter.next().is_some() {
                panic!("Ambiguous option {}", name);
            }
            n = first.1;
        }
    }

    pub fn insert(&mut self, name: &str, t: T) {
        let n = name.chars().fold(self, |n, c| n.children.entry(c).or_insert(NameTrie::new()));
        if n.t.is_some() {
            panic!("Collision in options at {}", name);
        }
        n.t = Some(t);
    }
}
