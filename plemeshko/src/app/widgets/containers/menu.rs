use std::marker::PhantomData;

use anyhow::Result;
use egui::{vec2, Button, RichText, Ui, Vec2, WidgetText};

use crate::app::{env::Env, widgets::Widget};

pub struct MenuAvailableSize<Id> {
    pub x: f32,
    pub y: f32,
    phantom: PhantomData<Id>,
}

impl<Id> MenuAvailableSize<Id> {
    pub fn vec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub trait MenuItem: Widget {
    fn scale(&self) -> Vec2;
}

pub struct Menu<Id> {
    pub layout: egui::Layout,
    pub items: Vec<Box<dyn MenuItem<Response = ()>>>,
    phantom: PhantomData<Id>,
}

impl<Id: 'static> Menu<Id> {
    pub fn new(layout: egui::Layout, items: Vec<Box<dyn MenuItem<Response = ()>>>) -> Self {
        Menu {
            layout,
            items,
            phantom: PhantomData,
        }
    }

    pub fn simple_scaled_text(
        text: impl Into<String>,
        font_size_base: f32,
        font_scaling: f32,
    ) -> impl Fn(&mut Env<'_>) -> WidgetText {
        let text: String = text.into();
        move |env| {
            let available_size = env.get::<MenuAvailableSize<Id>>().unwrap();
            let font_size = font_size_base + font_scaling * available_size.y;
            WidgetText::RichText(RichText::new(text.clone()).size(font_size))
        }
    }
}

impl<Id: 'static> Widget for Menu<Id> {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut Ui) -> anyhow::Result<Self::Response> {
        let win_size = ui.available_size();
        let mut menu_scale = Vec2::ZERO;
        for item in &self.items {
            let item_scale = item.scale();
            if self.layout.is_vertical() {
                menu_scale.y += item_scale.y;
                menu_scale.x = menu_scale.x.max(item_scale.x);
            } else {
                menu_scale.x += item_scale.x;
                menu_scale.y = menu_scale.y.max(item_scale.y);
            }
        }
        ui.with_layout(self.layout, |ui| {
            if self.layout.is_vertical() {
                ui.add_space(0.5 * win_size.y * (1.0 - menu_scale.y));
            } else {
                ui.add_space(0.5 * win_size.x * (1.0 - menu_scale.x));
            }
            env.with(
                &MenuAvailableSize {
                    x: win_size.x,
                    y: win_size.y,
                    phantom: self.phantom,
                },
                |env| {
                    for item in &mut self.items {
                        item.ui(env, ui)?;
                    }
                    Ok(())
                },
            )
        })
        .inner
    }
}

pub struct ScaledMenuItemBlank<Id> {
    pub scale: Vec2,
    phantom: PhantomData<Id>,
}

impl<Id> ScaledMenuItemBlank<Id> {
    pub fn new(scale: Vec2) -> Self {
        ScaledMenuItemBlank {
            scale,
            phantom: PhantomData,
        }
    }
}

impl<Id: 'static> Widget for ScaledMenuItemBlank<Id> {
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut Ui) -> anyhow::Result<()> {
        let available_size = env.get::<MenuAvailableSize<Id>>().unwrap();
        ui.allocate_space(self.scale * available_size.vec2());
        Ok(())
    }
}

impl<Id: 'static> MenuItem for ScaledMenuItemBlank<Id> {
    fn scale(&self) -> Vec2 {
        self.scale
    }
}

pub struct ScaledMenuItemButton<T, F, Id> {
    pub scale: Vec2,
    pub text: T,
    pub callback: F,
    phantom: PhantomData<Id>,
}

impl<T: FnMut(&mut Env<'_>) -> WidgetText, F: FnMut(&mut Env<'_>) -> Result<()>, Id>
    ScaledMenuItemButton<T, F, Id>
{
    pub fn new(scale: Vec2, text: T, callback: F) -> Self {
        ScaledMenuItemButton {
            scale,
            text,
            callback,
            phantom: PhantomData,
        }
    }
}

impl<T: FnMut(&mut Env<'_>) -> WidgetText, F: FnMut(&mut Env<'_>) -> Result<()>, Id: 'static> Widget
    for ScaledMenuItemButton<T, F, Id>
{
    type Response = ();

    fn ui(&mut self, env: &mut Env<'_>, ui: &mut Ui) -> Result<()> {
        let available_size = env.get::<MenuAvailableSize<Id>>().unwrap();
        if ui
            .add_sized(
                self.scale * available_size.vec2(),
                Button::new((self.text)(env)),
            )
            .clicked()
        {
            (self.callback)(env)?;
        }
        Ok(())
    }
}

impl<T: FnMut(&mut Env<'_>) -> WidgetText, F: FnMut(&mut Env<'_>) -> Result<()>, Id: 'static>
    MenuItem for ScaledMenuItemButton<T, F, Id>
{
    fn scale(&self) -> Vec2 {
        self.scale
    }
}

pub struct ScaledMenuItemTextEdit<Id> {
    pub scale: Vec2,
    pub text: String,
    pub multiline: bool,
    pub hint: RichText,
    phantom: PhantomData<Id>,
}

impl<Id: 'static> ScaledMenuItemTextEdit<Id> {
    pub fn new(scale: Vec2) -> Self {
        ScaledMenuItemTextEdit {
            scale,
            text: String::new(),
            multiline: false,
            hint: RichText::new(String::new()),
            phantom: PhantomData,
        }
    }
}

impl<Id: 'static> Widget for ScaledMenuItemTextEdit<Id> {
    type Response = ();

    fn ui(&mut self, _env: &mut Env<'_>, ui: &mut Ui) -> Result<Self::Response> {
        let te = if self.multiline {
            egui::TextEdit::multiline(&mut self.text)
        } else {
            egui::TextEdit::singleline(&mut self.text)
        };
        let te = te.hint_text(self.hint.clone());
        ui.add_sized(self.scale, te);
        Ok(())
    }
}

impl<Id: 'static> MenuItem for ScaledMenuItemTextEdit<Id> {
    fn scale(&self) -> Vec2 {
        self.scale
    }
}
