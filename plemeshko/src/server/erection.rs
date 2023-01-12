use plegine::config::ConfigRepository;

use crate::{cor::Cor, server::units::ResourceAmount};
use std::{
    collections::{hash_map::RawEntryMut, HashMap},
    ops::{AddAssign, SubAssign},
};

use super::{
    config::{
        method::{Method, MethodId, SelectedMethod},
        resource::{signed_storage::ResourceStorageSigned, storage::ResourceStorage, ResourceId},
        transport::{Transport, TransportId},
        transport_group::TransportGroupId,
    },
    error::{SimError, SimResult},
    units::ResourceWeight,
    Sim,
};

pub struct Erection {
    name: String,
    selected_methods: Vec<SelectedMethod>,
    transport: HashMap<TransportGroupId, TransportId>,
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
    pub fn new(
        sim: &Sim,
        name: String,
        selected_methods: Vec<SelectedMethod>,
        transport: HashMap<TransportGroupId, TransportId>,
    ) -> SimResult<Self> {
        let mut single_import = HashMap::<ResourceId, ResourceAmount>::new();
        let mut single_export = HashMap::<ResourceId, ResourceAmount>::new();
        let mut max_import = HashMap::new();
        let mut max_export = HashMap::new();
        for selected_method in selected_methods.iter() {
            // let method = sim.configs.get(&selected_method.id).map_err(SimError::ConfigRetrievalFailed)?;
            for selected_setting in selected_method.settings.iter() {
                let setting_group = sim
                    .configs
                    .get(&selected_setting.group_id)
                    .map_err(SimError::ConfigRetrievalFailed)?;
                let setting = &setting_group.settings[selected_setting.index];
                for (resource_id, delta) in setting.output.iter() {
                    single_export
                        .entry(resource_id.to_owned())
                        .or_default()
                        .add_assign(*delta);
                    let _ = max_export.try_insert(resource_id.to_owned(), ResourceAmount(0));
                }
                for (resource_id, delta) in setting.input.iter() {
                    single_import
                        .entry(resource_id.to_owned())
                        .or_default()
                        .sub_assign(*delta);
                    let _ = max_import.try_insert(resource_id.to_owned(), ResourceAmount(0));
                }
            }
        }

        Ok(Erection {
            name,
            selected_methods,
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn methods(&self) -> &Vec<SelectedMethod> {
        &self.selected_methods
    }

    pub fn transport(&self) -> &HashMap<TransportGroupId, TransportId> {
        &self.transport
    }

    //pub fn methods<'a>(&'a self, sim: &'a Sim) -> impl Iterator<Item = SimResult<&Method>> {
    //    iter_methods(&self.method_ids, sim)
    //}

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

    fn step_import(
        &mut self,
        depot: &mut ResourceStorage,
        configs: &ConfigRepository,
    ) -> SimResult<()> {
        let mut transport_state = HashMap::<TransportGroupId, (&Transport, ResourceWeight)>::new();
        let mut requested_resources = Vec::with_capacity(self.max_io.negative.len());
        for (res_id, req_amount) in self.max_io.negative.iter() {
            let res = configs
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
            match transport_state
                .raw_entry_mut()
                .from_key(&res.transport_group)
            {
                RawEntryMut::Vacant(vacant) => {
                    let tr = configs
                        .get(self.transport.get(&res.transport_group).unwrap())
                        .map_err(SimError::ConfigRetrievalFailed)?;
                    vacant.insert(res.transport_group.clone(), (tr, ResourceWeight(0)));
                }
                _ => (),
            }
        }

        requested_resources
            .sort_unstable_by_key(|(_, _, _, transportation_priority)| *transportation_priority);

        for (res_id, res, req_amount, _) in requested_resources {
            let tr_group = &res.transport_group;
            let (tr, tr_remaining) = transport_state.get_mut(tr_group).unwrap();
            let mut total_stored = ResourceAmount::default();
            let mut req_amount = req_amount;
            'a: while req_amount > ResourceAmount::default() {
                {
                    let amount_ready =
                        req_amount.min(ResourceAmount(*tr_remaining / res.transport_weight));
                    let res_depot = match depot.get_mut(res_id) {
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
                    if !depot.cor_sub_all(&tr.fuel.negative) {
                        break 'a;
                    }
                    depot.cor_put_all(&tr.fuel.positive);
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
            .cor_sub_all_times(&self.single_io.negative, self.active as i64);
        self.storage
            .positive
            .cor_put_all_times(&self.single_io.positive, activated);
    }

    // todo: fair export scheduler
    fn step_export(
        &mut self,
        depot: &mut ResourceStorage,
        configs: &ConfigRepository,
    ) -> SimResult<()> {
        let mut transport_state = HashMap::<TransportGroupId, (&Transport, ResourceWeight)>::new();
        for (res_id, res_amount) in self.storage.positive.iter_mut() {
            let res = configs
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
                let can_fuel = depot.cor_has_all_times(&tr.fuel.negative, i64::MAX);
                let req_transport = res_weight.0.div_ceil(tr.capacity.0);
                let transport = req_transport.min(can_fuel);
                depot.cor_sub_all_times_unchecked(&tr.fuel.negative, transport);
                depot.cor_put_all_times(&tr.fuel.positive, transport);
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
            match depot.raw_entry_mut().from_key(res_id) {
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

    pub fn step(
        &mut self,
        depot: &mut ResourceStorage,
        configs: &ConfigRepository,
    ) -> SimResult<()> {
        self.step_import(depot, configs)?;
        self.step_process();
        self.step_export(depot, configs)
    }
}

pub type ErectionContainer = Vec<Erection>;
