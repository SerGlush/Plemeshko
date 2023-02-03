use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use crate::state::{components::SharedComponents, AppState};

pub struct Env<'a>(HashMap<TypeId, &'a dyn Any>);

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env(HashMap::new())
    }

    pub fn get<'b, T: 'static>(&self) -> Option<&'b T>
    where
        'a: 'b,
    {
        self.0
            .get(&TypeId::of::<T>())
            .map(|any| any.downcast_ref::<T>().unwrap())
    }

    pub fn with<'v, T: 'static, R>(
        &mut self,
        value: &'v T,
        f: impl for<'f> FnOnce(&'f mut Env<'v>) -> R,
    ) -> R
    where
        'a: 'v,
    {
        let key = TypeId::of::<T>();
        // SAFETY:
        // May be unsafe when adding references with small lifetimes downstream
        // without deletion before casting to bigger lifetime.
        // Safe because the only way to insert to `Env` is using `with`
        // which removes the inserted ref immediately.
        let this = unsafe { std::mem::transmute::<_, &mut Env<'v>>(self) };
        if this.0.insert(key, value).is_some() {
            log::error!(
                "`Env::with` failed: Type already stored: {}",
                type_name::<T>()
            );
        }
        let result = f(this);
        this.0.remove(&key).unwrap();
        result
    }

    // Shortcuts, may panic

    pub fn shared_components(&self) -> &SharedComponents {
        self.get::<SharedComponents>().unwrap()
    }

    pub fn app_state(&self) -> &AppState {
        self.get::<AppState>().unwrap()
    }
}
