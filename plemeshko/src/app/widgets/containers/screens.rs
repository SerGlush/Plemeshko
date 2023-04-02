use std::marker::PhantomData;

use enum_map::{EnumArray, EnumMap};
use tap::TapOptional;

use crate::app::{env::Env, events::SetEvent, widgets::Widget};

pub struct Screens<R, E: EnumArray<Screen<R>>, Id> {
    current: E,
    screens: EnumMap<E, Screen<R>>,
    phantom: PhantomData<Id>,
}

pub struct ScreenTransitionEvent<E, Id>(SetEvent<E>, PhantomData<Id>);

type Screen<R> = Box<dyn Widget<Response = R>>;

impl<R, E: EnumArray<Screen<R>>, Id> Screens<R, E, Id> {
    pub fn new_at(initial: E, screens: EnumMap<E, Screen<R>>) -> Self {
        Screens {
            current: initial,
            screens,
            phantom: PhantomData,
        }
    }

    pub fn new(screens: EnumMap<E, Screen<R>>) -> Self
    where
        E: Default,
    {
        Self::new_at(E::default(), screens)
    }
}

impl<E, Id> ScreenTransitionEvent<E, Id> {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self, value: E);
            pub fn emit_idem(&self, value: E) -> Option<E>;
        }
    }

    pub fn new() -> Self {
        ScreenTransitionEvent(SetEvent::new(), PhantomData)
    }
}

impl<R, E: EnumArray<Screen<R>> + Copy + 'static, Id: 'static> Widget for Screens<R, E, Id> {
    type Response = R;

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut egui::Ui) -> anyhow::Result<Self::Response> {
        let ev_transition = ScreenTransitionEvent::<_, Id>::new();
        let result = env.with(&ev_transition, |env| self.screens[self.current].ui(env, ui));
        ev_transition.0.get().tap_some(|&next| self.current = next);
        result
    }
}
