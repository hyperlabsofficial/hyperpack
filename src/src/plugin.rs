pub trait Plugin {
    fn on_resolve(&self, file_path: &str) -> Option<String>;
    fn on_load(&self, file_path: &str, content: &str) -> Option<String>;
    fn on_transform(&self, file_path: &str, content: &str) -> Option<String>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn resolve(&self, file_path: &str) -> Option<String> {
        for plugin in &self.plugins {
            if let Some(new_path) = plugin.on_resolve(file_path) {
                return Some(new_path);
            }
        }
        None
    }

    pub fn load(&self, file_path: &str, content: &str) -> Option<String> {
        for plugin in &self.plugins {
            if let Some(new_content) = plugin.on_load(file_path, content) {
                return Some(new_content);
            }
        }
        None
    }

    pub fn transform(&self, file_path: &str, content: &str) -> Option<String> {
        for plugin in &self.plugins {
            if let Some(new_content) = plugin.on_transform(file_path, content) {
                return Some(new_content);
            }
        }
        None
    }
}