use alloc::collections::BTreeMap;
use crate::position::{WithSpan, Span}; // Import the WithSpan type from position.rs

impl<'a> IntoIterator for &'a HierarchicalName {
    type Item = (&'a WithSpan<String>, &'a WithSpan<String>);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.scopes.iter().flat_map(|map| map.iter()))
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct HierarchicalName {
    scopes: Vec<BTreeMap<WithSpan<String>, WithSpan<String>>>,
}

impl HierarchicalName {
    pub fn new() -> Self {
        HierarchicalName {
            scopes: vec![BTreeMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn add_path(&mut self, key: &WithSpan<String>, path: &WithSpan<String>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(key.clone(), path.clone());
        }
    }

    pub fn get_path(&self, key: &WithSpan<String>) -> Option<WithSpan<String>> {
        for scope in self.scopes.iter().rev() {
            if let Some(path) = scope.get(key) {
                return Some(path.clone());
            }
        }
        None
    }
    pub fn resolve_composite_key(&self, key: &WithSpan<String>) -> Option<WithSpan<String>> {
        let mut start = 0;
        let mut parts = Vec::new();
    
        for (end, _) in key.value.match_indices("::") {
            let part = &key.value[start..end];
            let span = Span::new_unchecked(key.span.start.0 + start as u32, end as u32 - start as u32);
            parts.push(WithSpan::new(part.to_string(), span));
            start = end + 2; // Move past the "::" separator
        }
    
        let last_part = &key.value[start..];
        let span = Span::new_unchecked(key.span.start.0 + start as u32, key.value.len() as u32 - start as u32);
        parts.push(WithSpan::new(last_part.to_string(), span));
    
        if parts.len() > 1 {
            for scope in self.scopes.iter().rev() {
                if let Some(value) = scope.get(&parts[0]) {
                    // Recursively resolve the remaining parts of the key
                    return self.resolve_composite_key(&WithSpan::new(
                        //format!("{}::{}", value.value, &parts[1..].join("::")),
                        format!("{}::{}", value.value, &parts[1..].iter().map(|s| s.value.as_str()).collect::<Vec<&str>>().join("::")),
                        Span::union(key, value)
                    ));
                }
            }
        }
        None
    }
    
}
