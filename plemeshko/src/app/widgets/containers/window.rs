use std::marker::PhantomData;

use anyhow::Result;
use egui::Ui;

use crate::app::{env::Env, events::FlagEvent, widgets::Widget};

pub trait PersistentWindowContent: Widget {
    fn title(&self, env: &Env<'_>) -> Result<egui::WidgetText>;
}

pub struct PersistentWindow<T, Id> {
    pub is_open: bool,
    pub content: T,
    phantom: PhantomData<Id>,
}

pub struct WindowCloseEvent<Id>(FlagEvent, PhantomData<Id>);

impl<T, Id> PersistentWindow<T, Id> {
    pub fn new(content: T) -> Self {
        PersistentWindow {
            is_open: false,
            content,
            phantom: PhantomData,
        }
    }
}

impl<Id> WindowCloseEvent<Id> {
    delegate::delegate! {
        to self.0 {
            pub fn emit(&self);
            pub fn emit_idem(&self) -> bool;
            pub fn get(&self) -> bool;
        }
    }

    pub fn new() -> Self {
        WindowCloseEvent(FlagEvent::new(), PhantomData)
    }
}

impl<T: PersistentWindowContent, Id: 'static> Widget for PersistentWindow<T, Id> {
    type Response = Option<T::Response>;

    fn ui(&mut self, env: &mut Env<'_>, _ui: &mut Ui) -> Result<Self::Response> {
        let egui_ctx = env.get::<egui::Context>().unwrap();
        let ev_close = WindowCloseEvent::<Id>::new();
        let result = env.with(&ev_close, |env| {
            if let Some(response) = egui::Window::new(self.content.title(env)?)
                .open(&mut self.is_open)
                .show(egui_ctx, |ui| self.content.ui(env, ui))
            {
                response.inner.transpose()
            } else {
                Ok(None)
            }
        });
        if ev_close.0.get() {
            self.is_open = false;
        }
        result
    }
}
