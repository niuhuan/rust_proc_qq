use lazy_static::lazy_static;
use proc_qq::Module;
use std::sync::Arc;

mod menu;
mod query;
mod tools;

lazy_static! {
    static ref MODULES: Arc<Vec<Module>> = Arc::new(vec![
        menu::module(),
        tools::group_admin::module(),
        query::image_lib::module(),
    ]);
}

pub(crate) fn all_modules() -> Arc<Vec<Module>> {
    MODULES.clone()
}
