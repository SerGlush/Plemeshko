use anyhow::Result;
use egui::Ui;

/// Consume iterator calling a fallible ui callback on each item.
/// Prepends each item's id with its index in the iterator.
pub fn draw_iter_indexed<T>(
    ui: &mut Ui,
    iter: impl Iterator<Item = T>,
    mut add_item_contents: impl FnMut(&mut Ui, T) -> Result<()>,
) -> Result<()> {
    for (index, item) in iter.enumerate() {
        ui.push_id(index, |ui| add_item_contents(ui, item)).inner?;
    }
    Ok(())
}
