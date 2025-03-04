use anyhow::Result;

mod helpers;
use crate::helpers::app::App;
use crate::helpers::audio::AudioEngine;

//Shows an error here sometimes but compiles fine, very cool.
slint::include_modules!();

#[allow(unused)]
fn main() -> Result<()> {
    let ae = AudioEngine::get_instance();
    let ae_init = ae.lock().unwrap().init()?;

    let app = App::run(Some(ae));
    if (app.is_err()) {
        return app;
    }

    return Ok(());
}
