mod actions;
mod audio;
mod device;
mod keys;
mod screenshot;
mod session_env;

use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use device::{resolve_project_root, run_device_supervisor};

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
    run_device_supervisor(project_root, &SHUTDOWN_REQUESTED)
}

fn install_shutdown_handler() -> Result<(), String> {
    ctrlc::set_handler(|| {
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
    })
    .map_err(|error| format!("failed to install ctrl-c handler: {error}"))
}
