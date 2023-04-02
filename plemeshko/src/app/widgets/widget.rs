use std::marker::PhantomData;

use anyhow::Result;
use egui::Ui;

use crate::app::env::Env;

pub trait Widget {
    type Response; // todo: default = ()   (blocked by incomplete unstable feature)

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut Ui) -> Result<Self::Response>;
}

impl Widget for () {
    type Response = ();

    fn ui(&mut self, _env: &mut Env<'_>, _ui: &mut Ui) -> Result<Self::Response> {
        Ok(())
    }
}

pub trait WidgetExt: Widget {
    fn map<R, F: FnMut(Self::Response) -> Result<R>>(self, f: F) -> WidgetMap<Self, R, F>
    where
        Self: Sized,
    {
        WidgetMap(self, f, PhantomData)
    }
}

pub struct WidgetMap<W, R, F>(pub W, pub F, PhantomData<R>);

impl<T: Widget> WidgetExt for T {}

impl<W: Widget, R, F: FnMut(W::Response) -> Result<R>> Widget for WidgetMap<W, R, F> {
    type Response = R;

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut Ui) -> Result<R> {
        self.1(self.0.ui(env, ui)?)
    }
}
