use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub width: T,
    pub height: T,
}

impl<T: Into<f32>> From<Rect<T>> for egui::Rect {
    fn from(value: Rect<T>) -> Self {
        let min_x = value.left.into();
        let min_y = value.top.into();
        let width = value.width.into();
        let height = value.height.into();
        egui::Rect {
            min: egui::pos2(min_x, min_y),
            max: egui::pos2(min_x + width, min_y + height),
        }
    }
}
