use colored::Colorize;
use directories::ProjectDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub template: String,
    pub target: String,
}

pub type Vars = HashMap<String, String>;
pub type Apps = HashMap<String, AppEntry>;

pub fn info(msg: impl AsRef<str>) {
    println!("colorctl({}): {}", "info".yellow(), msg.as_ref().yellow());
}
pub fn error(msg: impl AsRef<str>) {
    eprintln!("colorctl({}): {}", "error".red(), msg.as_ref().red());
}

pub fn cfg_dir() -> io::Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "colorctl")
        .ok_or_else(|| io::Error::other("cannot resolve config dir"))?;
    let path = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn read_json<T: for<'de> Deserialize<'de>>(p: &Path, default: T) -> io::Result<T> {
    match File::open(p) {
        Ok(mut f) => {
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            if s.trim().is_empty() {
                Ok(default)
            } else {
                serde_json::from_str(&s).map_err(io::Error::other)
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(default),
        Err(e) => Err(e),
    }
}

pub fn write_json<T: Serialize>(p: &Path, v: &T) -> io::Result<()> {
    let mut f = File::create(p)?;
    let s = serde_json::to_string_pretty(v).unwrap();
    f.write_all(s.as_bytes())
}

pub fn expand(p: &str) -> String {
    shellexpand::tilde(p).to_string()
}

pub fn load_state() -> io::Result<(Vars, Apps, PathBuf, PathBuf)> {
    let dir = cfg_dir()?;
    let vars_p = dir.join("variables.json");
    let apps_p = dir.join("applications.json");
    let vars: Vars = read_json(&vars_p, HashMap::new())?;
    let apps: Apps = read_json(&apps_p, HashMap::new())?;
    Ok((vars, apps, vars_p, apps_p))
}

pub fn save_vars(vars_p: &Path, vars: &Vars) {
    if let Err(e) = write_json(vars_p, vars) {
        error(format!("failed saving variables: {e}"));
    }
}

pub fn save_apps(apps_p: &Path, apps: &Apps) {
    if let Err(e) = write_json(apps_p, apps) {
        error(format!("failed saving applications: {e}"));
    }
}

pub fn apply(vars: &Vars, apps: &Apps) {
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
