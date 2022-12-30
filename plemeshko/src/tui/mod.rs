use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use cursive::{
    view::{IntoBoxedView, Nameable, Resizable},
    views, Cursive,
};

use crate::sim::{depot::Depot, Sim};

struct Tui {}

impl Default for Tui {
    fn default() -> Self {
        Tui {}
    }
}

fn depot_ui_init<S: Into<String>>(name: S) -> impl IntoBoxedView + 'static {
    views::ListView::new().with_name(name).fixed_width(20)
}

fn depot_ui_update(
    siv: &mut Cursive,
    name: &str,
    /* assets: &Assets, */ depot: &Depot,
) -> Option<()> {
    siv.call_on_name(name, |list: &mut views::ListView| {
        list.clear();
        for (id, count) in depot.iter() {
            list.add_child(id.as_str(), views::TextView::new(count.to_string()));
        }
    })
}

pub fn run(sim: Arc<Mutex<Sim>>) {
    let mut siv = cursive::default();
    let tui = Rc::new(RefCell::new(Tui::default()));

    {
        let sim = sim.clone();
        siv.add_global_callback('q', move |siv| {
            sim.lock().unwrap().set_quit();
            siv.quit()
        });
    }

    let layer = views::LinearLayout::horizontal()
        .child(depot_ui_init("main_depot"))
        .child(views::Button::new("Fetch", move |siv| {
            let sim_lock = sim.lock().unwrap();
            // let mut tui = tui.borrow_mut();
            let depot = sim_lock.get_main_depot();
            depot_ui_update(siv, "main_depot", depot).unwrap();
        }));
    siv.add_layer(layer);

    siv.run();
}
