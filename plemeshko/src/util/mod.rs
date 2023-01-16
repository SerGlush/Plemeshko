pub mod cor;

macro_rules! config_get {
    ($cfgs:expr, $id:expr) => {
        anyhow::Context::with_context($cfgs.get($id), (|| "Config retrieval failed"))?
    };
}
