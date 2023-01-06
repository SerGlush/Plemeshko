use crate::{cor::Cor, sim::units::ResourceAmount};
use std::{
    collections::{
        hash_map::{Entry, RawEntryMut},
        HashMap,
    },
    ops::{AddAssign, SubAssign},
};

use super::{
    config::{
        method::{Method, MethodId},
        resource::{signed_storage::ResourceStorageSigned, ResourceId},
        transport::{Transport, TransportId, TransportMap},
    },
    error::{SimError, SimResult},
    transport_group::TransportGroup,
    units::ResourceWeight,
    Sim,
};

pub struct Erection {
    method_ids: Vec<MethodId>,
    transport: TransportMap<TransportId>,
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

// todo: storage can be initialized with zeroes for known i/o; at all accesses presence of known keys can be then guaranteed

impl Erection {
    pub fn new(sim: &Sim, method_ids: Vec<MethodId>) -> SimResult<Self> {
        let mut total_delta = HashMap::<ResourceId, ResourceAmount>::new();
        for method in iter_methods(&method_ids, sim) {
            let method = method?;
            for (resource_id, delta) in method.resources.positive.iter() {
                total_delta
                    .entry(resource_id.to_owned())
                    .or_default()
                    .add_assign(*delta);
            }
            for (resource_id, delta) in method.resources.negative.iter() {
                total_delta
                    .entry(resource_id.to_owned())
                    .or_default()
                    .sub_assign(*delta);
            }
        }

        let mut transport = HashMap::<TransportGroup, TransportId>::new();
        for (resource_id, delta) in total_delta.iter() {
            if *delta != ResourceAmount::default() {
                let tg = sim
                    .configs
                    .get(resource_id)
                    .map_err(SimError::ConfigRetrievalFailed)?
                    .transport_group;
                let _ = transport.try_insert(tg, sim.default_transport(tg));
            }
        }

        let mut single_import = HashMap::with_capacity(total_delta.len() / 2);
        let mut single_export = HashMap::with_capacity(total_delta.len() / 2);
        let mut max_import = HashMap::with_capacity(total_delta.len() / 2);
        let mut max_export = HashMap::with_capacity(total_delta.len() / 2);
        for (resource_id, delta) in total_delta {
            if delta < ResourceAmount::default() {
                single_import.insert(resource_id.clone(), -delta);
                max_import.insert(resource_id, ResourceAmount(0));
            } else if delta > ResourceAmount::default() {
                single_export.insert(resource_id.clone(), delta);
                max_export.insert(resource_id, ResourceAmount(0));
            }
        }

        Ok(Erection {
            method_ids,
            transport,
            single_io: ResourceStorageSigned {
                positive: single_export,
                negative: single_import,
            },
            count: 0,
            active: 0,
            max_io: ResourceStorageSigned {
                positive: max_export,
                negative: max_import,
            },
            storage: ResourceStorageSigned::new(),
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

    fn step_import(&mut self, sim: &mut Sim) -> SimResult<()> {
        let mut transport_state = TransportMap::<(&Transport, ResourceWeight)>::new();
        let mut requested_resources = Vec::with_capacity(self.max_io.negative.len());
        for (res_id, req_amount) in self.max_io.negative.iter() {
            let res = sim
                .configs
                .get(res_id)
                .map_err(SimError::ConfigRetrievalFailed)?;
            let already_stored = self
                .storage
                .negative
                .get(res_id)
                .map(Clone::clone)
                .unwrap_or_default();
            if already_stored >= *req_amount {
                continue;
            }
            let req_amount_transported = *req_amount - already_stored;
            let transportation_priority =
                req_amount_transported / *self.single_io.negative.get(res_id).unwrap();
            requested_resources.push((
                res_id,
                res,
                req_amount_transported,
                transportation_priority,
            ));
            match transport_state.entry(res.transport_group) {
                Entry::Vacant(vacant) => {
                    let tr = sim
                        .configs
                        .get(self.transport.get(&res.transport_group).unwrap())
                        .map_err(SimError::ConfigRetrievalFailed)?;
                    vacant.insert((tr, ResourceWeight(0)));
                }
                _ => (),
            }
        }

        requested_resources
            .sort_unstable_by_key(|(_, _, _, transportation_priority)| *transportation_priority);

        for (res_id, res, req_amount, _) in requested_resources.iter() {
            let tr_group = res.transport_group;
            let (tr, tr_remaining) = transport_state.get_mut(&tr_group).unwrap();
            let mut total_stored = ResourceAmount::default();
            let mut req_amount = *req_amount;
            'a: while req_amount > ResourceAmount::default() {
                {
                    let amount_ready =
                        req_amount.min(ResourceAmount(*tr_remaining / res.transport_weight));
                    let res_depot = match sim.depot.get_mut(res_id) {
                        Some(res_depot) => res_depot,
                        None => break 'a, // todo: handle 0-required-res ?
                    };
                    if *res_depot == ResourceAmount::default() {
                        break 'a;
                    }
                    let amount_ready = amount_ready.min(*res_depot);
                    tr_remaining.sub_assign(amount_ready * res.transport_weight);
                    req_amount.sub_assign(amount_ready);
                    total_stored += amount_ready;
                }
                while *tr_remaining < res.transport_weight {
                    if !sim.depot.cor_sub_all(&tr.fuel.negative) {
                        break 'a;
                    }
                    sim.depot.cor_put_all(&tr.fuel.positive);
                }
            }
            self.storage.negative.cor_put(res_id, total_stored);
        }
        Ok(())
    }

    fn step_process(&mut self) {
        let activated = self
            .storage
            .negative
            .cor_sub_all_times(&self.single_io.negative, self.active as i128);
        self.storage
            .positive
            .cor_put_all_times(&self.single_io.positive, activated);
    }

    // todo: fair export scheduler
    fn step_export(&mut self, sim: &mut Sim) -> SimResult<()> {
        let mut transport_state = TransportMap::<(&Transport, ResourceWeight)>::new();
        for (res_id, res_amount) in self.storage.positive.iter_mut() {
            let res = sim
                .configs
                .get(res_id)
                .map_err(SimError::ConfigRetrievalFailed)?;
            let (tr, transported_weight_already) =
                transport_state.get_mut(&res.transport_group).unwrap();
            let mut res_weight = *res_amount * res.transport_weight;
            let transported_amount = if res_weight <= *transported_weight_already {
                transported_weight_already.sub_assign(res_weight);
                *res_amount
            } else {
                res_weight.sub_assign(*transported_weight_already);
                let transported_amount_already =
                    ResourceAmount(*transported_weight_already / res.transport_weight);
                *transported_weight_already = ResourceWeight(0);
                let can_fuel = sim.depot.cor_has_all_times(&tr.fuel.negative, i128::MAX);
                let req_transport = res_weight.0.div_ceil(tr.capacity.0);
                let transport = req_transport.min(can_fuel);
                sim.depot
                    .cor_sub_all_times_unchecked(&tr.fuel.negative, transport);
                sim.depot.cor_put_all_times(&tr.fuel.positive, transport);
                let transported_amount_newly = ResourceAmount(
                    (*res_amount - transported_amount_already)
                        .0
                        .min((tr.capacity * transport) / res.transport_weight),
                );
                let transported_amount_total =
                    transported_amount_already + transported_amount_newly;
                let transported_weight_newly = tr.capacity * transport;
                let transported_weight_newly_used = transported_amount_newly * res.transport_weight;
                let transported_weight_newly_remain =
                    transported_weight_newly - transported_weight_newly_used;
                transported_weight_already.add_assign(transported_weight_newly_remain);
                transported_amount_total
            };
            // update storages
            res_amount.sub_assign(transported_amount);
            match sim.depot.raw_entry_mut().from_key(res_id) {
                RawEntryMut::Occupied(mut occupied) => {
                    occupied.get_mut().add_assign(transported_amount)
                }
                RawEntryMut::Vacant(vacant) => {
                    vacant.insert(res_id.clone(), transported_amount);
                }
            }
        }
        Ok(())
    }

    pub fn step(&mut self, sim: &mut Sim) -> SimResult<()> {
        self.step_import(sim)?;
        self.step_process();
        self.step_export(sim)
    }
}

pub type ErectionContainer = Vec<Erection>;
