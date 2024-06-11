
use std::collections::HashMap;

#[derive(Debug)]
struct HierarchicalName {
    scopes: Vec<HashMap<String, String>>,
}

impl HierarchicalName {
    fn new() -> Self {
        HierarchicalName {
            scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_path(&mut self, key: &str, path: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(key.to_string(), path.to_string());
        }
    }

    fn get_path(&self, key: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(path) = scope.get(key) {
                return Some(path.clone());
            }
        }
        None
    }

    fn resolve_composite_key(&self, composite_key: &str) -> Option<String> {
        let keys: Vec<&str> = composite_key.split("::").collect();
        let mut resolved_path = String::new();
        
        for key in keys {
            if let Some(value) = self.get_path(key) {
                if !resolved_path.is_empty() {
                    resolved_path.push_str("::");
                }
                resolved_path.push_str(&value);
            } else {
                return None; // Return None if any key is not found
            }
        }
        
        Some(resolved_path)
    }
}

#[derive(Debug)]
struct UrlResolver {
    hierarchical_name: HierarchicalName,
}

impl UrlResolver {
    fn new() -> Self {
        UrlResolver {
            hierarchical_name: HierarchicalName::new(),
        }
    }

    fn push_scope(&mut self) {
        self.hierarchical_name.push_scope();
    }

    fn pop_scope(&mut self) {
        self.hierarchical_name.pop_scope();
    }

    fn resolve_url(&mut self, key: &str, url: &str) {
        // Extract the path from the URL and add it to HierarchicalName
        let path = self.extract_path_from_url(url);
        self.hierarchical_name.add_path(key, &path);
    }

    fn extract_path_from_url(&self, url: &str) -> String {
        url.to_string()
    }

    fn get_path_for_key(&self, key: &str) -> Option<String> {
        if let Some(value) = self.hierarchical_name.get_path(key) {
            if value.contains("::") {
                // The value is a composite key, resolve it
                self.hierarchical_name.resolve_composite_key(&value)
            } else {
                Some(value)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_scoped_url_resolver_test() {
        let mut url_resolver = UrlResolver::new();

        // Global scope
        url_resolver.resolve_url("public_key", "sio79f708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abd");
        url_resolver.resolve_url("type", "src");
        url_resolver.resolve_url("name", "app_name");
        url_resolver.resolve_url("app", "public_key::type::name");

        // General block scope
        url_resolver.push_scope();
        url_resolver.resolve_url("g0", "siopub00119a708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abdabca");
        url_resolver.resolve_url("g1", "siopub00129a708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abdabcb");
        
        assert_eq!(
            url_resolver.get_path_for_key("g0"),
            Some("siopub00119a708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abdabca".to_string())
        );
        assert_eq!(
            url_resolver.get_path_for_key("g1"),
            Some("siopub00129a708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abdabcb".to_string())
        );

        // Pop the general block scope
        url_resolver.pop_scope();

        assert_eq!(
            url_resolver.get_path_for_key("public_key"),
            Some("sio79f708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abd".to_string())
        );
        assert_eq!(
            url_resolver.get_path_for_key("app"),
            Some("sio79f708c25a23ed367610facc14035adc7ba4b1bfa9252ef55c6c24f1b9b03abd::src::app_name".to_string())
        );
    }
}
