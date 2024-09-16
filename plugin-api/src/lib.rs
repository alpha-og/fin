pub trait Plugin {
    fn init(&self);
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}
