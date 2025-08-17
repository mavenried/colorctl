use clap::{CommandFactory, Parser};
use colored::Colorize;
use directories::ProjectDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod cli;
use cli::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppEntry {
    template: String,
    target: String,
}
type Vars = HashMap<String, String>;
type Apps = HashMap<String, AppEntry>;

fn info(msg: impl AsRef<str>) {
    println!("colorctl({}): {}", "info".yellow(), msg.as_ref().yellow());
}
fn error(msg: impl AsRef<str>) {
    eprintln!("colorctl({}): {}", "error".red(), msg.as_ref().red());
}

fn cfg_dir() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "colorctl")
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot resolve config dir"))?;
    let path = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn read_json<T: for<'de> Deserialize<'de>>(p: &Path, default: T) -> io::Result<T> {
    match File::open(p) {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            if s.trim().is_empty() {
                Ok(default)
            } else {
                serde_json::from_str(&s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(default),
        Err(e) => Err(e),
    }
}

fn write_json<T: Serialize>(p: &Path, v: &T) -> io::Result<()> {
    let mut f = File::create(p)?;
    let s = serde_json::to_string_pretty(v).unwrap();
    f.write_all(s.as_bytes())
}

fn expand(p: &str) -> String {
    shellexpand::tilde(p).to_string()
}

fn load_state() -> io::Result<(Vars, Apps, PathBuf, PathBuf)> {
    let dir = cfg_dir()?;
    let vars_p = dir.join("variables.json");
    let apps_p = dir.join("applications.json");
    let vars: Vars = read_json(&vars_p, HashMap::new())?;
    let apps: Apps = read_json(&apps_p, HashMap::new())?;
    Ok((vars, apps, vars_p, apps_p))
}

fn save_vars(vars_p: &Path, vars: &Vars) {
    if let Err(e) = write_json(vars_p, vars) {
        error(format!("failed saving variables: {e}"));
    }
}

fn save_apps(apps_p: &Path, apps: &Apps) {
    if let Err(e) = write_json(apps_p, apps) {
        error(format!("failed saving applications: {e}"));
    }
}

fn apply(vars: &Vars, apps: &Apps) {
    let re = Regex::new(r#"\$\[([^\]]+)\]"#).unwrap();
    for (name, app) in apps {
        let template_path = expand(&app.template);
        let target_path = expand(&app.target);

        let template = match fs::read_to_string(&template_path) {
            Ok(s) => s,
            Err(_) => {
                error(format!(
                    "no such template `{}` for app {name}",
                    app.template
                ));
                continue;
            }
        };

        let mut unknowns: Vec<String> = vec![];
        let out = re
            .replace_all(&template, |caps: &regex::Captures| {
                let key = &caps[1];
                if let Some(val) = vars.get(key) {
                    info(format!("found var `{key}` in {name} conf"));
                    val.to_string()
                } else {
                    unknowns.push(key.to_string());
                    String::new()
                }
            })
            .to_string();

        for u in unknowns {
            error(format!("found unknown var `{u}` in {name} conf"));
        }

        if let Err(e) = fs::write(&target_path, out) {
            error(format!("write failed for `{}`: {e}", target_path));
        }
    }
}

fn vars_ctl(op: &str, vars: &mut Vars) {
    if let Some(stripped) = op.strip_prefix('+') {
        if let Some((k, v)) = stripped.split_once('=') {
            info(format!("set `{k}` to {v}"));
            vars.insert(k.to_string(), v.to_string());
        } else {
            error("use +name=value");
        }
    } else if let Some(name) = op.strip_prefix('-') {
        if vars.remove(name).is_none() {
            error(format!("no such variable `{name}`"));
        } else {
            info(format!("Removed {name}."))
        }
    } else {
        error("unknown vars operation");
    }
}

fn apps_ctl(op: &str, apps: &mut Apps) {
    if let Some(stripped) = op.strip_prefix('+') {
        // Format: +app=template,target  (matches the original Python code behavior)
        if let Some((appname, value)) = stripped.split_once('=') {
            if let Some((template, target)) = value.split_once(',') {
                info(format!(
                    "added `{appname}` with-\n template: {template}\n target: {target}"
                ));
                apps.insert(
                    appname.to_string(),
                    AppEntry {
                        template: template.to_string(),
                        target: target.to_string(),
                    },
                );
            } else {
                error("use +app=template,target");
            }
        } else {
            error("use +app=template,target");
        }
    } else if let Some(appname) = op.strip_prefix('-') {
        if apps.remove(appname).is_none() {
            error(format!("no such app `{appname}`"));
        } else {
            info(format!("Removed {appname}."));
        }
    } else {
        error("unknown apps operation");
    }
}

fn pick_with_fzf(options: impl IntoIterator<Item = String>) -> io::Result<Option<String>> {
    let input = options.into_iter().collect::<Vec<_>>().join("\n");
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
    }
    let out = child.wait_with_output()?;
    if out.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

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
