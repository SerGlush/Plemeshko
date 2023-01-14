pub mod cor;

macro_rules! config_get {
    ($cfgs:expr, $id:expr) => {
        $cfgs
            .get($id)
            .with_context(|| "Config retrieval failed: {e}")?
    };
}
