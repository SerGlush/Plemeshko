pub mod cor;

macro_rules! config_get {
    ($cfgs:expr, $id:expr) => {
        anyhow::Context::with_context($cfgs.get($id), (|| "Config retrieval failed"))?
    };
}

pub fn display_each<E: std::fmt::Display>(
    f: &mut std::fmt::Formatter<'_>,
    es: impl Iterator<Item = E>,
    delim: &str,
    stop: &str,
) -> std::fmt::Result {
    let mut comma = false;
    for e in es {
        if comma {
            write!(f, "{delim}")?;
        }
        e.fmt(f)?;
        comma = true;
    }
    write!(f, "{stop}")
}
