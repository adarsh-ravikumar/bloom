use std::collections::{HashMap, HashSet};

use crate::module::{Module, ModuleMemberType, ModuleMetadata, ModuleType};

#[derive(Debug)]
pub struct ModuleRegistry {
    pub lookup: HashMap<String, usize>,
    modules: Vec<Module>,
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

    pub fn lookup(
        &mut self,
        module_name: String,
        attribute_name: String,
    ) -> Option<&ModuleMemberType> {
        if let Some(id) = self.lookup.get(&module_name) {
            self.modules.get(*id).unwrap().metadata.get(&attribute_name)
        } else {
            None
        }
    }

    fn visit_module(
        &'a self,
        module_name: &'a String,
        visited: HashSet<&'a String>,
        completed: &mut HashMap<&'a String, usize>,
    ) {
        println!("visiting {module_name}. path {visited:?}");

        if let Some(count) = completed.get_mut(&module_name) {
            *count += 1;
            return;
        }

        let index = self.lookup.get(module_name);

        if index.is_none() {
            println!("Module not found: {module_name}");
            return;
        }

        let module = &self.modules[*index.unwrap()];

        for (dependency, _) in &module.dependencies {
            if visited.contains(dependency) {
                println!(
                    "CIRCULAR DEPENDENCY DETECTED ON MODULE {dependency:?}! {visited:?}"
                );
            } else {
                let mut visited = visited.clone();
                visited.insert(dependency);

                self.visit_module(dependency, visited, completed);
            }
        }

        completed.insert(module_name, 0);
    }

    pub fn analyze_dependencies(&self) {
        let mut completed = HashMap::new();

        for module in self.lookup.keys() {
            let mut visited = HashSet::new();
            visited.insert(module);

            self.visit_module(module, visited, &mut completed);
        }

        for (module_name, count) in completed {
            let module = &self.modules[*self.lookup.get(module_name).unwrap()];

            if count == 0 && matches!(module.r#type, ModuleType::Widget) {
                println!("Dead widget: {module_name}");
            }
        }
    }
}
