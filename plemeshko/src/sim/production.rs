use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    sim::units::ResourceAmount,
    state::{
        components::{ComponentsRef, SharedComponents},
        config::FatConfigLabel,
        serializable::Serializable,
        SharedState,
    },
    util::cor::Cor,
};
use std::{
    collections::{
        hash_map::{Entry, RawEntryMut},
        HashMap,
    },
    ops::{AddAssign, SubAssign},
};

use super::{
    config::{
        production_method::{FixedProductionMethod, RawFixedProductionMethod},
        resource::{RawResourceMap, ResourceId, ResourceIo, ResourceMap},
        transport_group::{TransportGroup, TransportGroupId},
        transport_method::{TransportMethod, TransportMethodId},
    },
    units::ResourceWeight,
};

#[derive(Serialize, Deserialize)]
pub struct RawProductionSnapshot {
    name: String,
    selected_methods: Vec<RawFixedProductionMethod>,
    transport: HashMap<FatConfigLabel<TransportGroup>, FatConfigLabel<TransportMethod>>,
    storage: RawResourceMap,
    count: u32,
    active: u32,
    reserve_export_threshold: u32,
}

#[derive(Clone)]
pub struct ProductionSnapshot {
    name: String,
    selected_methods: Vec<FixedProductionMethod>,
    transport: HashMap<TransportGroupId, TransportMethodId>,
    storage: ResourceMap,
    count: u32,
    active: u32,
    reserve_export_threshold: u32,
}

pub struct Production {
    state: ProductionSnapshot,
    single_io: ResourceIo,
}

// todo: storage can be initialized with zeroes for known i/o; at all accesses presence of known keys can be then guaranteed

impl Production {
    pub fn new(
        shared_comps: &SharedComponents,
        name: String,
        selected_methods: Vec<FixedProductionMethod>,
        transport: HashMap<TransportGroupId, TransportMethodId>,
    ) -> anyhow::Result<Self> {
        Self::restore(
            shared_comps,
            ProductionSnapshot {
                name,
                selected_methods,
                transport,
                storage: ResourceMap::new(),
                count: 1,
                active: 1,
                reserve_export_threshold: 1,
            },
        )
    }

    pub fn restore(
        shared_comps: &SharedComponents,
        snapshot: ProductionSnapshot,
    ) -> anyhow::Result<Self> {
        let mut single_input = HashMap::<ResourceId, ResourceAmount>::new();
        let mut single_output = HashMap::<ResourceId, ResourceAmount>::new();
        for selected_method in snapshot.selected_methods.iter() {
            for &setting_id in selected_method.settings.iter() {
                let setting = shared_comps.config(setting_id)?;
                for (resource_id, delta) in setting.resource_io.output.iter() {
                    single_output
                        .entry(resource_id.to_owned())
                        .or_default()
                        .add_assign(*delta);
                }
                for (resource_id, delta) in setting.resource_io.input.iter() {
                    single_input
                        .entry(resource_id.to_owned())
                        .or_default()
                        .add_assign(*delta);
                }
            }
        }

        Ok(Production {
            state: snapshot,
            single_io: ResourceIo {
                output: single_output,
                input: single_input,
            },
        })
    }

    pub fn snapshot(&self) -> ProductionSnapshot {
        self.state.clone()
    }

    pub fn name(&self) -> &str {
        &self.state.name
    }

    pub fn methods(&self) -> &Vec<FixedProductionMethod> {
        &self.state.selected_methods
    }

    pub fn transport(&self) -> &HashMap<TransportGroupId, TransportMethodId> {
        &self.state.transport
    }

    /// production building size
    pub fn count(&self) -> u32 {
        self.state.count
    }

    pub fn active(&self) -> u32 {
        self.state.active
    }

    pub fn set_count(&mut self, count: u32) {
        if count < self.state.active {
            self.set_active(self.state.count);
        }
        self.state.count = count;
    }

    pub fn set_active(&mut self, active: u32) {
        self.state.active = active;
    }

    pub fn io(&self) -> &ResourceIo {
        &self.single_io
    }

    fn step_input(
        &mut self,
        shared_st: &SharedState,
        depot: &mut ResourceMap,
    ) -> anyhow::Result<()> {
        let shared_comps = shared_st.components.read().unwrap();
        let mut transport_state =
            HashMap::<TransportGroupId, (&TransportMethod, ResourceWeight)>::new();
        let mut requested_resources = Vec::with_capacity(self.single_io.input.len());
        for (res_id, &single_input) in self.single_io.input.iter() {
            let req_input = single_input * self.active() as i64;
            let res = shared_comps.config(*res_id)?;
            let already_stored = self
                .state
                .storage
                .get(res_id)
                .map(Clone::clone)
                .unwrap_or_default();
            if already_stored >= req_input {
                continue;
            }
            let req_import = req_input - already_stored;
            let transportation_priority =
                req_import.div_ceil(*self.single_io.input.get(res_id).unwrap());
            requested_resources.push((res_id, res, req_import, transportation_priority));
            if let RawEntryMut::Vacant(vacant) = transport_state
                .raw_entry_mut()
                .from_key(&res.transport_group)
            {
                let tr_id = *self.state.transport.get(&res.transport_group).unwrap();
                let tr = shared_comps.config(tr_id)?;
                vacant.insert(res.transport_group, (tr, ResourceWeight(0)));
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
                    let amount_ready = if res.transport_weight.0 != 0 {
                        req_amount.min(ResourceAmount(*tr_remaining / res.transport_weight))
                    } else {
                        req_amount
                    };
                    let res_depot = match depot.get_mut(res_id) {
                        Some(res_depot) => res_depot,
                        None => break 'a, // todo: handle 0-required-res ?
                    };
                    if *res_depot == ResourceAmount::default() {
                        break 'a;
                    }
                    let amount_ready = amount_ready.min(*res_depot);
                    res_depot.sub_assign(amount_ready);
                    tr_remaining.sub_assign(amount_ready * res.transport_weight);
                    req_amount.sub_assign(amount_ready);
                    total_stored += amount_ready;
                }
                if req_amount == ResourceAmount(0) {
                    break;
                }
                if req_amount < ResourceAmount(0) {
                    panic!("Negative amount");
                };
                let tr_required_add_weight = res.transport_weight - *tr_remaining;
                let tr_required_add_count = tr_required_add_weight.div_ceil(tr.capacity);
                if depot.cor_has_all_times(&tr.fuel.input, tr_required_add_count)
                    != tr_required_add_count
                {
                    break;
                }
                depot.cor_sub_all_times_unchecked(&tr.fuel.input, tr_required_add_count);
                //# TODO: don't add output until the end of the step (ex: fuel = non-consumed vehicle)
                // opt: normalized step stages? like "post-step"
                depot.cor_put_all_times(&tr.fuel.output, tr_required_add_count);
                tr_remaining.add_assign(tr.capacity * tr_required_add_count);

                // while *tr_remaining < res.transport_weight {
                //     if !depot.cor_sub_all(&tr.fuel.input) {
                //         break 'a;
                //     }
                //     depot.cor_put_all(&tr.fuel.output);
                // }
            }
            self.state.storage.cor_put(res_id, total_stored);
        }
        Ok(())
    }

    fn step_process(&mut self) {
        let activated = self
            .state
            .storage
            .cor_sub_all_times(&self.single_io.input, self.state.active as i64);
        self.state
            .storage
            .cor_put_all_times(&self.single_io.output, activated);
    }

    // todo: fair output scheduler
    fn step_output(
        &mut self,
        shared_st: &SharedState,
        depot: &mut ResourceMap,
    ) -> anyhow::Result<()> {
        let shared_comps = shared_st.components.read().unwrap();
        let mut transport_state =
            HashMap::<TransportGroupId, (&TransportMethod, ResourceWeight)>::new();
        let active = self.active() as i64;
        for (&res_id, res_amount) in self.state.storage.iter_mut() {
            // humans are always exported back to the global storage
            if self.state.reserve_export_threshold > 0 && res_id != shared_st.human_id {
                // other resources are exported when above the reserve limit
                if let Some(&single_input) = self.single_io.input.get(&res_id) {
                    let tick_input = single_input * active;
                    if *res_amount < tick_input * self.state.reserve_export_threshold as i64 {
                        continue;
                    }
                }
            }
            let res = shared_comps.config(res_id)?;
            let (tr, transported_weight_already) = match transport_state
                .raw_entry_mut()
                .from_key(&res.transport_group)
            {
                RawEntryMut::Vacant(vacant) => {
                    let tr_id = *self.state.transport.get(&res.transport_group).unwrap();
                    let tr = shared_comps.config(tr_id)?;
                    vacant
                        .insert(res.transport_group, (tr, ResourceWeight(0)))
                        .1
                }
                RawEntryMut::Occupied(occupied) => occupied.into_mut(),
            };
            let mut res_weight = *res_amount * res.transport_weight;
            let transported_amount = if res_weight <= *transported_weight_already {
                transported_weight_already.sub_assign(res_weight);
                *res_amount
            } else {
                res_weight.sub_assign(*transported_weight_already);
                let transported_amount_already =
                    ResourceAmount(*transported_weight_already / res.transport_weight);
                *transported_weight_already = ResourceWeight(0);
                let can_fuel = depot.cor_has_all_times(&tr.fuel.input, i64::MAX);
                let req_transport = res_weight.div_ceil(tr.capacity);
                let transport = req_transport.min(can_fuel);
                depot.cor_sub_all_times_unchecked(&tr.fuel.input, transport);
                depot.cor_put_all_times(&tr.fuel.output, transport);
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
            match depot.entry(res_id) {
                Entry::Occupied(mut occupied) => occupied.get_mut().add_assign(transported_amount),
                Entry::Vacant(vacant) => {
                    vacant.insert(transported_amount);
                }
            }
        }
        Ok(())
    }

    pub fn step(&mut self, env: &SharedState, depot: &mut ResourceMap) -> anyhow::Result<()> {
        self.step_input(env, depot)?;
        self.step_process();
        self.step_output(env, depot)
    }
}

impl Serializable for ProductionSnapshot {
    type Raw = RawProductionSnapshot;

    fn from_serializable(raw: Self::Raw, ctx: ComponentsRef<'_>) -> Result<Self> {
        Ok(ProductionSnapshot {
            name: raw.name,
            selected_methods: Serializable::from_serializable(raw.selected_methods, ctx)?,
            transport: Serializable::from_serializable(raw.transport, ctx)?,
            storage: Serializable::from_serializable(raw.storage, ctx)?,
            count: raw.count,
            active: raw.active,
            reserve_export_threshold: raw.reserve_export_threshold,
        })
    }

    fn into_serializable(self, ctx: ComponentsRef<'_>) -> Result<Self::Raw> {
        Ok(RawProductionSnapshot {
            name: self.name,
            selected_methods: self.selected_methods.into_serializable(ctx)?,
            transport: self.transport.into_serializable(ctx)?,
            storage: self.storage.into_serializable(ctx)?,
            count: self.count,
            active: self.active,
            reserve_export_threshold: self.reserve_export_threshold,
        })
    }
}
