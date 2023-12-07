pub mod cli;
mod db;
pub mod err_print;
pub mod state;
pub mod xrandr;

use std::{process::Child, str::FromStr};

use clap::Parser;
use cli::Cli;
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{randr::ConnectionExt as RandrExt, screensaver::ConnectionExt as ScreensaverExt},
};

use crate::{err_print::ErrPrint, state::State, xrandr::XrandrCmd};

pub fn exec_on_remap(on_remap: Option<&String>) -> anyhow::Result<Option<Child>> {
    let on_remap = on_remap.as_ref().and_then(|on_remap| {
        let split = shlex::split(on_remap)
            .err_print(format!("Failed to parse on_remap command: `{on_remap:?}`",))?;
        if split.is_empty() {
            None
        } else {
            let mut cmd = std::process::Command::new(split[0].clone());
            for arg in &split[1..] {
                cmd.arg(arg.clone());
            }
            Some(cmd)
        }
    });
    if let Some(mut cmd) = on_remap {
        Ok(Some(cmd.spawn()?))
    } else {
        Ok(None)
    }
}

fn main_loop(cli: Cli) -> anyhow::Result<()> {
    let lock_path = proc_lock::LockPath::FullPath("/tmp/autoxrandr.lock");
    let _lock = proc_lock::try_lock(&lock_path).map_err(|_| {
        anyhow::anyhow!("Cannot acquire lock. Is another instance of autoxrandr running?")
    })?;
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .or(std::env::var("HOME").map(|home| format!("{home}/.cache")))
        .expect("Cannot find cache dir");
    let mut cache_dir = std::path::PathBuf::from_str(&cache_dir)?;
    cache_dir.push(env!("CARGO_PKG_NAME"));
    std::fs::create_dir_all(&cache_dir)?;

    let db = db::AutoXrandrDB::new(&cache_dir)?;

    let (conn, screen_num) = x11rb::connect(cli.display.as_deref())?;
    let screen = &conn.setup().roots[screen_num];
    let outputs = conn
        .randr_get_screen_resources(screen.root)?
        .reply()?
        .outputs;
    println!("Autoxrandr started");
    let mut process_handle = None;
    if cli.reapply {
        let state = State::current(&conn, screen.root, &outputs)?;
        if let Ok(previous) = db.get_state(&state.outputs_sign()) {
            XrandrCmd::from(&previous).exec()?;
            process_handle = exec_on_remap(cli.on_remap.as_ref())?;
        }
    }
    conn.extension_information(x11rb::protocol::randr::X11_EXTENSION_NAME)?
        .ok_or_else(|| anyhow::anyhow!("Cannot find randr extension"))?;
    conn.extension_information(x11rb::protocol::screensaver::X11_EXTENSION_NAME)?
        .ok_or_else(|| anyhow::anyhow!("Cannot find screensaver extension"))?;
    loop {
        // We respect the screensaver state, so we don't mess with it.
        if let Ok(screen_cookie) = conn.screensaver_query_info(screen.root) {
            if let Ok(screensaver) = screen_cookie.reply() {
                let screensaver_state =
                    x11rb::protocol::screensaver::State::from(screensaver.state);
                if screensaver_state == x11rb::protocol::screensaver::State::ON
                    || screensaver_state == x11rb::protocol::screensaver::State::CYCLE
                {
                    std::thread::sleep(std::time::Duration::from_secs(cli.delay));
                    continue;
                }
            }
        }
        if let Ok(state) = State::current(&conn, screen.root, &outputs) {
            if state.should_map() || state.should_unmap() {
                let mut was_remapped = false;
                let previous_state = db.get_state(&state.outputs_sign());
                if let Ok(previous_state) = previous_state {
                    match XrandrCmd::from(&previous_state)
                        .exec()
                        .err_print("Cannot restore previous state".into())
                    {
                        Ok(_) => {
                            was_remapped = true;
                        }
                        Err(_) => {
                            db.remove_state(&previous_state)
                                .err_print("Cannot remove previous state.".into())
                                .ok();
                        }
                    }
                } else if state.should_unmap() {
                    XrandrCmd::from(&state)
                        .exec()
                        .err_print("Cannot unmap screens".into())
                        .ok();
                    was_remapped = true;
                }
                if was_remapped {
                    if let Some(previous) = process_handle.as_mut() {
                        println!("Killing previous process");
                        previous.kill()?;
                    }
                    if let Ok(handle) = exec_on_remap(cli.on_remap.as_ref())
                        .err_print("Failed to execute on_remap command while unmapping.".into())
                    {
                        process_handle = handle;
                    }
                }
            } else {
                db.save_state(&state)
                    .err_print("Cannot save current state.".into())
                    .ok();
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(cli.delay));
    }
}

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    if cli.background {
        if let Ok(daemon) = fork::daemon(true, false) {
            match daemon {
                fork::Fork::Parent(_) => {
                    println!("Autoxrandr started in background");
                }
                fork::Fork::Child => {
                    main_loop(cli)?;
                }
            }
        }
        return Ok(());
    } else {
        main_loop(cli)?;
    }
    Ok(())
}
