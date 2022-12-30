use crate::units::Mass;
use std::collections::HashMap;

use super::{
    config::{
        method::{Method, MethodId},
        resource::{delta_resources, ResourceId},
        transport::TransportId,
    },
    error::{SimError, SimResult},
    transport_group::TransportGroup,
    Sim,
};

pub struct Erection {
    method_ids: Vec<MethodId>,
    transport: HashMap<TransportGroup, TransportId>,
    import: Vec<(ResourceId, Mass, Mass)>, // id, for active=1, stored
    export: Vec<(ResourceId, Mass, Mass)>, // id, for active=1, stored
    count: u32,
    active: u32,
}

fn iter_methods<'a>(
    method_ids: &'a Vec<MethodId>,
    sim: &'a Sim,
) -> impl Iterator<Item = SimResult<&'a Method>> {
    method_ids
        .iter()
        .map(|id| sim.configs.get(id).map_err(SimError::ConfigRetrievalFailed))
}

impl Erection {
    pub fn new(sim: &Sim, method_ids: Vec<MethodId>) -> SimResult<Self> {
        // Import & export
        let mut total_delta = Vec::new();
        for method_id in method_ids {
            sim.configs
                .get(&method_id)
                .map_err(SimError::ConfigRetrievalFailed)?
                .accumulate(&mut total_delta);
        }
        let mut import = Vec::with_capacity(total_delta.len() / 2);
        let mut export = Vec::with_capacity(total_delta.len() / 2);
        for (resource_id, delta) in total_delta {
            if delta < Mass(0) {
                import.push((resource_id, -delta, Mass(0)));
            } else if delta > Mass(0) {
                export.push((resource_id, delta, Mass(0)));
            }
        }

        // Default transport
        let mut transport = HashMap::<TransportGroup, TransportId>::new();
        for method in iter_methods(&method_ids, sim) {
            for resource in delta_resources(sim, &method?.delta) {
                let tg = resource?.0.transportation_group;
                let _ = transport.try_insert(tg, sim.default_transport(tg));
            }
        }

        Ok(Erection {
            method_ids,
            transport,
            import,
            export,
            count: 0,
            active: 0,
        })
    }

    pub fn methods<'a>(&'a self, sim: &'a Sim) -> impl Iterator<Item = SimResult<&Method>> {
        iter_methods(&self.method_ids, sim)
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn active(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) {
        if count < self.active {
            self.set_active(self.count);
        }
        self.count = count;
    }

    pub fn set_active(&mut self, active: u32) {
        // let delta = active - self.active;
        self.active = active;
    }

    pub fn step(&mut self, sim: &mut Sim) {
        // determine needed transport to make stored-import+transported>=required-import*active
        // determine needed fuel
        // consume min(available, needed) fuel
        // transfer resources to the import storage accordingly
        // determine how many times (<= active) process can run, needed resources
        // consume min(available, needed) resources
        // produce results accordingly, store them in the export storage
        // determine needed transport to transfer stored export
        // determine needed fuel
        // consume min(available, needed) fuel
        // transfer resources from the export storage accordingly
        todo!();
    }
}

pub type ErectionContainer = Vec<Erection>;
