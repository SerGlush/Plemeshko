use crate::state::config::ConfigTypeRegistry;

pub mod method_group;
pub mod production_method;
pub mod resource;
pub mod setting;
pub mod setting_group;
pub mod transport_group;
pub mod transport_method;

pub fn register() -> anyhow::Result<ConfigTypeRegistry> {
    let mut reg = ConfigTypeRegistry::new();
    reg.register::<production_method::Method>()?;
    reg.register::<method_group::MethodGroup>()?;
    reg.register::<resource::Resource>()?;
    reg.register::<setting::Setting>()?;
    reg.register::<setting_group::SettingGroup>()?;
    reg.register::<transport_method::Transport>()?;
    reg.register::<transport_group::TransportGroup>()?;
    Ok(reg)
}
