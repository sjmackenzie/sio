use crate::frontend::hierarchical_name::HierarchicalName;
use crate::frontend::position::{WithSpan};
use alloc::format;
use alloc::string::String;
use crate::alloc::string::ToString;
use alloc::vec::Vec;
#[derive(Debug)]
pub struct UrlResolver {
    hierarchical_name: HierarchicalName,
}

impl UrlResolver {
    pub fn new() -> Self {
        UrlResolver {
            hierarchical_name: HierarchicalName::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.hierarchical_name.push_scope();
    }

    pub fn pop_scope(&mut self) {
        self.hierarchical_name.pop_scope();
    }

    pub fn resolve_url(&mut self, key: &WithSpan<String>, url: &WithSpan<String>) {
        let path = WithSpan::new(url.value.clone(), url.span);
        self.hierarchical_name.add_path(key, &path);
    }

    pub fn extract_path_from_url(&self, url: &str) -> String {
        url.to_string()
    }

    pub fn get_path_for_key(&self, key: &WithSpan<String>) -> Option<WithSpan<String>> {
        let iter = self.hierarchical_name.into_iter().collect::<Vec<_>>(); // Convert into Vec
        for (k, v) in iter.into_iter().rev() { // Convert back to iterator and use rev
            if k == key {
                return Some(v.clone());
            }
        }
        None
    }

    pub fn flatten_source(&self, source_code: &str) -> String {
        let mut result = source_code.to_string();
        let iter = self.hierarchical_name.into_iter().collect::<Vec<_>>(); // Convert into Vec
    
        for (key, value) in iter.into_iter() { // Convert back to iterator
            if let Some(composite_value) = self.hierarchical_name.resolve_composite_key(value) {
                result = result.replace(&format!("{}::", key), &format!("{}::", composite_value));
            } else {
                result = result.replace(&format!("{}::", key), &format!("{}::", value));
            }
        }
        result
    }
}
