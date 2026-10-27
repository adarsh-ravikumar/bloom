use std::collections::{HashMap, HashSet, hash_map::Keys};

use crate::module::{
    self, Module, ModuleMemberType, ModuleMetadata, ModuleType,
};

#[derive(Debug)]
pub struct ModuleRegistry {
    lookup: HashMap<String, usize>,
    pub modules: Vec<Module>,
}

impl<'a> ModuleRegistry {
    pub fn new() -> Self {
        Self {
            lookup: HashMap::new(),
            modules: Vec::new(),
        }
    }

    pub fn register(&mut self, name: String, module: Module) {
        self.lookup.insert(name, self.modules.len());
        self.modules.push(module);
    }

    pub fn modules(&self) -> Keys<String, usize> {
        self.lookup.keys()
    }

    pub fn get_module(&self, module_name: &String) -> Option<&Module> {
        if let Some(id) = self.lookup.get(module_name) {
            self.modules.get(*id)
        } else {
            None
        }
    }

    // pub fn lookup_attribute(
    //     &self,
    //     module_name: &String,
    //     attribute_name: &String,
    // ) -> Option<&ModuleMemberType> {
    //     if let Some(module) = self.get_module(&module_name) {
    //     } else {
    //         None
    //     }
    // }

    pub fn analyze_dependencies(&self) -> Vec<usize> {
        let mut completed = HashMap::new();
        let mut dependency_list = Vec::new();

        for module in self.lookup.values() {
            let mut visited = HashSet::new();
            visited.insert(*module);

            self.visit_module(
                *module,
                visited,
                &mut completed,
                &mut dependency_list,
            );
        }

        for (module, count) in completed {
            let module = &self.modules[module];

            if count == 0 && matches!(module.r#type, ModuleType::Widget) {
                println!("Dead widget: {module:?}");
            }
        }

        println!("{:?}", self.lookup);
        dependency_list
    }

    fn visit_module(
        &'a self,
        module_index: usize,
        visited: HashSet<usize>,
        completed: &mut HashMap<usize, usize>,
        dependency_list: &mut Vec<usize>,
    ) {
        // println!("visiting {module_index}. path {visited:?}");

        if let Some(count) = completed.get_mut(&module_index) {
            // permanent mark.
            *count += 1;
            return;
        }

        let module = &self.modules[module_index];

        for (dependency, _) in &module.dependencies {
            let index = if let Some(index) = self.lookup.get(dependency) {
                *index
            } else {
                println!("Module not found {dependency}");
                continue;
            };

            if visited.contains(&index) {
                println!(
                    "CIRCULAR DEPENDENCY DETECTED ON MODULE {dependency:?}! {visited:?}"
                );
            } else {
                let mut visited = visited.clone();

                // temporary mark
                visited.insert(index);

                self.visit_module(index, visited, completed, dependency_list);
            }
        }

        completed.insert(module_index, 0);
        dependency_list.push(module_index);
    }

    // visit module
}
