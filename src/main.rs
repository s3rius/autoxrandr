pub mod cli;
mod db;
pub mod err_print;
pub mod state;
pub mod xrandr;

use std::str::FromStr;

use clap::Parser;
use x11rb::connection::Connection;

use crate::{err_print::ErrPrint, state::State};

pub fn exec_on_remap(on_remap: Option<&String>) -> anyhow::Result<()> {
    let on_remap = on_remap.as_ref().and_then(|on_remap| {
        let split = shlex::split(on_remap).err_print(format!(
            "Failed to parse on_remap command: `{:?}`",
            on_remap
        ))?;
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
        cmd.spawn()?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .or(std::env::var("HOME").map(|home| format!("{home}/.cache")))
        .expect("Cannot find cache dir");
    let mut cache_dir = std::path::PathBuf::from_str(&cache_dir)?;
    cache_dir.push(env!("CARGO_PKG_NAME"));
    std::fs::create_dir_all(&cache_dir)?;

    let db = db::AutoRandrDB::new(&cache_dir)?;

    let (conn, screen_num) = x11rb::connect(cli.display.as_ref().map(|screen| screen.as_str()))?;
    let screen = &conn.setup().roots[screen_num];

    println!("Autoxrandr started");
    if cli.reapply {
        let state = State::from_screen(&conn, screen)?;
        if let Ok(previous) = db.get_state(&state.outputs_sign()) {
            previous.to_xrandr_cmd().exec()?;
            exec_on_remap(cli.on_remap.as_ref())?;
        }
    }
    loop {
        if let Ok(state) = State::from_screen(&conn, screen) {
            let mut should_map = state.should_map();
            if state.should_unmap() {
                state
                    .to_xrandr_cmd()
                    .exec()
                    .err_print("Cannot unmap screens".into())
                    .ok();
                exec_on_remap(cli.on_remap.as_ref())
                    .err_print("Failed to execute on_remap command.".into())
                    .ok();
                should_map = true;
            }
            if should_map {
                let previous_state = db.get_state(&state.outputs_sign());
                if let Ok(previous_state) = previous_state {
                    match previous_state
                        .to_xrandr_cmd()
                        .exec()
                        .err_print("Cannot restore previous state".into())
                    {
                        Ok(()) => {
                            exec_on_remap(cli.on_remap.as_ref())
                                .err_print("Failed to execute on_remap command.".into())
                                .ok();
                        }
                        Err(_) => {
                            db.remove_state(&previous_state)
                                .err_print("Cannot remove previous state.".into())
                                .ok();
                        }
                    }
                }
            } else {
                db.save_state(&state)
                    .err_print("Cannot save current state.".into())
                    .ok();
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
