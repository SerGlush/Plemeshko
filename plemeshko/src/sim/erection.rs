use crate::sim::units::ResourceAmount;
use std::{collections::{HashMap, hash_map::{RawEntryMut, Entry}}, ops::{AddAssign, SubAssign}};

use super::{
    config::{
        method::{Method, MethodId},
        resource::{ResourceId, signed_storage::ResourceStorageSigned},
        transport::{TransportId, TransportMap, Transport},
    },
    error::{SimError, SimResult},
    transport_group::TransportGroup,
    Sim, units::{TransportAmount, ResourceWeight},
};

pub struct ErectionTransport {
    id: TransportId,
    // max_amount: TransportAmount,
    single_weight: ResourceWeight,
}

pub struct Erection {
    method_ids: Vec<MethodId>,
    transport: TransportMap<ErectionTransport>,
    single_io: ResourceStorageSigned,
    max_io: ResourceStorageSigned,
    storage: ResourceStorageSigned,
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
        let mut total_delta = HashMap::<ResourceId, ResourceAmount>::new();
        for method in iter_methods(&method_ids, sim) {
            for (resource_id, delta) in method?.resources.positive.iter() {
                total_delta.entry(resource_id.to_owned()).or_default().add_assign(*delta);
            }
            for (resource_id, delta) in method?.resources.negative.iter() {
                total_delta.entry(resource_id.to_owned()).or_default().sub_assign(*delta);
            }
        }

        let mut transport = HashMap::<TransportGroup, TransportId>::new();
        for (resource_id, delta) in total_delta.iter() {
            if *delta != ResourceAmount::default() {
                let tg = sim.configs.get(resource_id).map_err(SimError::ConfigRetrievalFailed)?.transport_group;
                let _ = transport.try_insert(tg, sim.default_transport(tg));
            }
        }

        let mut import = Vec::with_capacity(total_delta.len() / 2);
        let mut export = Vec::with_capacity(total_delta.len() / 2);
        for (resource_id, delta) in total_delta {
            if delta < ResourceAmount::default() {
                import.push((resource_id, -delta, ResourceAmount::default()));
            } else if delta > ResourceAmount::default() {
                export.push((resource_id, delta, ResourceAmount::default()));
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

    pub fn step(&mut self, sim: &mut Sim) -> SimResult<()> {
        let mut planned_input = self.max_io.negative.clone();
        let todo_erections = self.active as i128;
        let todo_erections = todo_erections - self.storage.negative.sub_bounded(&self.single_io.negative, todo_erections);
        if todo_erections == 0 {
            return Ok(());
        }

        let req_tr_weight = TransportMap::<ResourceWeight>::with_capacity(self.transport.len());
        for (res_id, mut res_amount) in self.single_io.negative.iter() {
            res_amount *= todo_erections;
            res_amount -= self.storage.negative.get(res_id).unwrap_or_default();
            let res = sim.configs.get(res_id).map_err(SimError::ConfigRetrievalFailed)?;
            req_tr_weight.entry(res.transport_group).or_default().add_assign(res.transport_weight * res_amount);
        }

        let transport = TransportMap::<&Transport>::with_capacity(self.transport.len());
        for (tr_group, _) in req_tr_weight.iter() {
            let tr_id = &self.transport.get(&tr_group).unwrap().id;
            let tr = sim.configs.get(tr_id).map_err(SimError::ConfigRetrievalFailed)?;
            transport.insert(*tr_group, tr);
        }

        let mut available_tr_weight = TransportMap::<ResourceWeight>::with_capacity(self.transport.len());
        'a: while(!req_tr_weight.is_empty()) {
            for tr_group in self.transport.keys() {
                let tr = transport.get(tr_group).unwrap();
                let tr_weight = req_tr_weight.entry(*tr_group);
                let tr_weight = match tr_weight {
                    Entry::Occupied(occupied) => occupied,
                    Entry::Vacant(_) => continue,
                };
                if !sim.depot.sub(&tr.fuel.negative) {
                    tr_weight.remove_entry();
                    break 'a;
                }
                let available_tr_weight_s = available_tr_weight.entry(*tr_group).or_default();
                available_tr_weight_s.add_assign(tr.capacity);
                if *available_tr_weight_s >= *tr_weight.get() {
                    break 'a;
                }
            }
        }

        // todo: transport resources from sim.depot to self.storage for all available available_tr_weight
        // then use all available resources to run process
        // all results go to the output storage
        // then they are transported to the depot


        // # old outline below
        // transfer resources to the import storage accordingly
        // determine how many times (<= active) process can run, needed resources
        // consume min(available, needed) resources
        // produce results accordingly, store them in the export storage
        // determine needed transport to transfer stored export
        // determine needed fuel
        // consume min(available, needed) fuel
        // transfer resources from the export storage accordingly
        Ok(())
    }
}

pub type ErectionContainer = Vec<Erection>;
