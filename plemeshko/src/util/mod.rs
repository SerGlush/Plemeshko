pub mod cor;

mod rect;

pub use rect::*;

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
