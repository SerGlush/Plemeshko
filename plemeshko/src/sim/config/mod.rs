use crate::state::config::ConfigTypeRegistry;

pub mod method;
pub mod method_group;
pub mod resource;
pub mod setting_group;
pub mod transport;
pub mod transport_group;

pub fn register() -> anyhow::Result<ConfigTypeRegistry> {
    let mut reg = ConfigTypeRegistry::new();
    reg.register::<resource::Resource>()?;
    reg.register::<setting_group::SettingGroup>()?;
    reg.register::<transport::Transport>()?;
    reg.register::<transport_group::TransportGroup>()?;
    reg.register::<method::Method>()?;
    reg.register::<method_group::MethodGroup>()?;
    Ok(reg)
}
