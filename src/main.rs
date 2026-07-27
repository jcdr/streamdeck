mod actions;
mod audio;
mod device;
mod keys;
mod screenshot;
mod session_env;

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use device::{resolve_project_root, DeckRuntime};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

fn main() -> ExitCode {
    match run_application() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run_application() -> Result<(), String> {
    install_shutdown_handler()?;
    session_env::apply_session_gui_environment()?;

    let project_root = resolve_project_root();
    let deck_runtime = Arc::new(DeckRuntime::open(project_root)?);
    deck_runtime.apply_key_images()?;
    deck_runtime.run_event_loop_until_shutdown(&SHUTDOWN_REQUESTED)?;

    if let Err(error) = deck_runtime.reset_device() {
        eprintln!("reset failed on shutdown: {error}");
    }

    Ok(())
}

fn install_shutdown_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
    })
    .map_err(|error| format!("failed to install ctrl-c handler: {error}"))
}
