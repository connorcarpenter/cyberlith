use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub struct DependencyMap<K: Hash + Copy + Eq + PartialEq, V> {
    // waiting key -> (dependencies, entry)
    dependent_map: HashMap<K, (HashSet<K>, V)>,
    // dependency -> key waiting on it
    dependency_map: HashMap<K, HashSet<K>>,
}

impl<K: Hash + Copy + Eq + PartialEq, V> DependencyMap<K, V> {
    pub fn new() -> Self {
        Self {
            dependent_map: HashMap::new(),
            dependency_map: HashMap::new(),
        }
    }

    pub fn insert_waiting_dependencies(
        &mut self,
        dependency_keys: Vec<K>,
        dependent_key: K,
        dependent_value: V,
    ) {
        if !self.dependent_map.contains_key(&dependent_key) {
            self.dependent_map
                .insert(dependent_key, (HashSet::new(), dependent_value));
        }

        for dependency_key in dependency_keys {
            if !self.dependency_map.contains_key(&dependency_key) {
                self.dependency_map
                    .insert(dependency_key, HashSet::new());
            }
            let dependents = self.dependency_map.get_mut(&dependency_key).unwrap();
            dependents.insert(dependent_key);

            let (dependencies, _) = self.dependent_map.get_mut(&dependent_key).unwrap();
            dependencies.insert(dependency_key);
        }
    }

    pub fn on_dependency_complete(&mut self, key: K) -> Option<Vec<(K, V)>> {
        if let Some(dependents) = self.dependency_map.remove(&key) {
            let mut result = Vec::new();
            for dependent in dependents {
                let (dependencies, _) = self.dependent_map.get_mut(&dependent).unwrap();
                dependencies.remove(&key);
                if dependencies.is_empty() {
                    let (_, entry) = self.dependent_map.remove(&dependent).unwrap();
                    result.push((dependent, entry));
                }
            }
            Some(result)
        } else {
            None
        }
    }
}
