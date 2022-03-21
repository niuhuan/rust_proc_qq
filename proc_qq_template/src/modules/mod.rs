use lazy_static::lazy_static;
use proc_qq::Module;
use std::sync::Arc;

mod game;
mod menu;
mod query;
mod tools;

lazy_static! {
    static ref MODULES: Arc<Vec<Module>> = Arc::new(vec![
        menu::module(),
        game::group_sign_in::module(),
        tools::group_admin::module(),
        query::image_lib::module(),
        query::daily_english::module(),
    ]);
}

pub(crate) fn all_modules() -> Arc<Vec<Module>> {
    MODULES.clone()
}

pub(crate) async fn init_modules() -> anyhow::Result<()> {
    game::group_sign_in::init_data_base().await?;
    Ok(())
}
