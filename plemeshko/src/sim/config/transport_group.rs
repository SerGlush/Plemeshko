use serde::Deserialize;

use crate::state::{
    components::{ComponentIndexer, SharedComponents},
    config::{Config, FatConfigId, Prepare},
    text::FatTextId,
};

use super::transport_method::{TransportMethod, TransportMethodId};

#[derive(Deserialize)]
pub struct RawTransportGroup {}

#[derive(Debug)]
pub struct TransportGroup {
    pub name: FatTextId,
    pub transports: Vec<TransportMethodId>,
}

pub type TransportGroupId = FatConfigId<TransportGroup>;

impl Prepare for RawTransportGroup {
    type Prepared = TransportGroup;

    fn prepare(
        self,
        ctx: &mut crate::state::config::ConfigsLoadingContext<'_>,
        tif: &mut crate::state::text::TextIdFactory,
    ) -> anyhow::Result<Self::Prepared> {
        let name = tif.create("name").in_component(ctx.this_component.id());
        Ok(TransportGroup {
            name,
            transports: Vec::new(),
        })
    }
}

impl Config for TransportGroup {
    type Raw = RawTransportGroup;

    const TAG: &'static str = "transport-group";

    fn finalize(
        indexer: &ComponentIndexer,
        shared_comps: &mut SharedComponents,
    ) -> anyhow::Result<()> {
        // clear all transport groups
        for transport_group in shared_comps.iter_configs_mut::<TransportGroup>() {
            let transport_group = transport_group?.1;
            transport_group.transports.clear();
        }

        // for all components - find all settings and push to respective groups
        let component_slot_ids = indexer.indices();
        for component_slot_id in component_slot_ids {
            let component_transport_ids = match shared_comps.component_slot(component_slot_id)? {
                Some(component) => component
                    .configs
                    .indexer::<TransportMethod>()?
                    .indices::<TransportMethod>(),
                None => continue,
            };
            let component_id = component_slot_id.assume_occupied();
            for component_transport_id in component_transport_ids {
                let transport_group_id = shared_comps
                    .component_slot(component_slot_id)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .configs
                    .storage::<TransportMethod>()?
                    .get(component_transport_id)?
                    .group;
                shared_comps
                    .config_mut(transport_group_id)?
                    .transports
                    .push(FatConfigId(component_id, component_transport_id));
            }
        }
        Ok(())
    }
}
