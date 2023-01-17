macro_rules! config_text_id {
    ($id:expr, $fmt:literal, $($rest:tt)*) => {
        crate::env::text::TextId::new(format!(concat!("{}_{}", $fmt), Self::TAG, $id, $($rest)*))
    };
    ($id:expr) => {
        config_text_id!($id, "",)
    };
}

pub mod method;
pub mod method_group;
pub mod resource;
pub mod setting_group;
pub mod transport;
pub mod transport_group;

pub fn register(builder: &mut crate::env::config::ConfigRepositoryBuilder) -> anyhow::Result<()> {
    builder.register::<resource::Resource>()?;
    builder.register::<setting_group::SettingGroup>()?;
    builder.register::<transport::Transport>()?;
    builder.register::<transport_group::TransportGroup>()?;
    builder.register::<method::Method>()?;
    builder.register::<method_group::MethodGroup>()?;
    Ok(())
}
