use anyhow::Result;
use std::sync::Arc;

mod helpers;
use crate::helpers::app::App;
use crate::helpers::audio::AudioEngine;

//Shows an error here sometimes but compiles fine, very cool.
slint::include_modules!();

#[allow(unused)]
fn main() -> Result<()> {
    let ae = Arc::new(AudioEngine::get_instance());
    let ae_init = ae.lock().unwrap().init()?;

    let app = App::run();
    if (app.is_err()) {
        return app;
    }

    return Ok(());
}
