use plegine::config::{ConfigRepo, ConfigRetrievalError};

use super::{config::{resource::{ResourceCount, ResourceVec}, method::MethodId}, depot::Depot};

pub struct ErectionClass {
    methods: Vec<MethodId>,
    total_delta: ResourceVec,
}

impl ErectionClass {
    pub fn new(cfgs: &ConfigRepo, methods: Vec<MethodId>) -> Result<ErectionClass, ConfigRetrievalError> {
        let mut total_delta = Vec::new();
        for method_id in methods {
            cfgs.get(&method_id)?.apply(&mut total_delta);
        }
        Ok(ErectionClass {
            methods,
            total_delta,
        })
    }

    pub fn methods(&self) -> impl Iterator<Item = &MethodId> {
        self.methods.iter()
    }
}

pub struct Erection {
    class: ErectionClass,
    count: u32,
    active: u32,
    total_delta: Vec<ResourceCount>,
}

impl Erection {
    pub fn new(class: ErectionClass) -> Self {
        let total_delta = Vec::with_capacity(class.total_delta.len());
        Erection {
            class,
            count: 0,
            active: 0,
            total_delta,
        }
    }

    pub fn class(&self) -> &ErectionClass {
        &self.class
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
        let delta: i64 = active as i64 - self.active as i64;
        for i in 0..self.total_delta.len() {
            self.total_delta[i] += delta as i128 * self.class.total_delta[i].1;
        }
    }
}

pub type ErectionContainer = Vec<Erection>;
