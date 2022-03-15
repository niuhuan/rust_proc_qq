use lazy_static::lazy_static;
use proc_qq::Module;
use std::sync::Arc;

mod group_admin;
mod imglib;
mod menu;

lazy_static! {
    static ref MODULES: Arc<Vec<Module>> = Arc::new(vec![
        menu::module(),
        imglib::module(),
        group_admin::module()
    ]);
}

pub(crate) fn all_modules() -> Arc<Vec<Module>> {
    MODULES.clone()
}
