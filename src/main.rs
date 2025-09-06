use clap::{CommandFactory, Parser};
use colored::Colorize;
use std::process::Command;

mod cli;
use cli::*;

mod handlers;
use handlers::*;

mod utility;
use utility::*;

fn main() {
    let cli = Cli::parse();

    // Load state
    let (mut vars, mut apps, vars_p, apps_p) = match load_state() {
        Ok(x) => x,
        Err(e) => {
            error(format!("failed to load state: {e}"));
            return;
        }
    };

    if let Some(cmd) = cli.cmd {
        match cmd {
            Cmd::Apply => apply(&vars, &apps),
            Cmd::Vars { op } => {
                if !op.is_empty() {
                    vars_ctl(&op.iter().fold(String::new(), |a, e| a + e), &mut vars);
                    save_vars(&vars_p, &vars);
                } else {
                    for (k, v) in &vars {
                        println!("{}\n -> {}", k.magenta(), v.yellow());
                    }
                }
            }
            Cmd::Apps { op } => {
                if !op.is_empty() {
                    apps_ctl(&op.iter().fold(String::new(), |a, e| a + e), &mut apps);
                    save_apps(&apps_p, &apps);
                } else {
                    for (name, a) in &apps {
                        println!("{} =>", name.magenta());
                        println!("   > Template -> {}", expand(&a.template).yellow());
                        println!("   > Target   -> {}", expand(&a.target).yellow());
                    }
                }
            }
            Cmd::Edit { apps: to_edit } => {
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string());

                if to_edit.is_empty() {
                    match pick_with_fzf(apps.keys().cloned()) {
                        Ok(Some(app)) => {
                            if let Some(entry) = apps.get(&app) {
                                let path = expand(&entry.template);
                                let status = Command::new(&editor).arg(&path).status();
                                match status {
                                    Ok(_) => {
                                        info(format!("opened {} in {}", entry.template, editor))
                                    }
                                    Err(e) => error(format!("failed to run editor: {e}")),
                                }
                            } else {
                                info("nothing picked!");
                            }
                        }
                        Ok(None) => info("nothing picked!"),
                        Err(e) => error(format!("fzf failed: {e}")),
                    }
                } else {
                    for app in to_edit {
                        if let Some(entry) = apps.get(&app) {
                            let path = expand(&entry.template);
                            let status = Command::new(&editor).arg(&path).status();
                            match status {
                                Ok(_) => info(format!("opened {} in {}", entry.template, editor)),
                                Err(e) => error(format!("failed to run editor: {e}")),
                            }
                        } else {
                            error("app does not exist.");
                        }
                    }
                }
            }
        }
    } else {
        Cli::command().print_help().unwrap();
        println!();
    }
}
