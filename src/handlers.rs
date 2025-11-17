use crate::utility::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};
pub fn vars_ctl(op: &str, vars: &mut Vars) {
    if let Some(stripped) = op.strip_prefix('+') {
        if let Some((k, v)) = stripped.split_once('=') {
            tracing::info!("Set `{k}` to {v}.");
            vars.insert(k.to_string(), v.to_string());
        } else {
            tracing::error!("Use +name=value");
        }
    } else if let Some(name) = op.strip_prefix('-') {
        if vars.remove(name).is_none() {
            tracing::error!("No such variable `{name}`.");
        } else {
            tracing::info!("Removed {name}.")
        }
    } else {
        tracing::error!("Unknown vars operation.");
    }
}

pub fn apps_ctl(op: &str, apps: &mut Apps) {
    if let Some(stripped) = op.strip_prefix('+') {
        if let Some((appname, value)) = stripped.split_once('=') {
            if let Some((template, target)) = value.split_once(',') {
                tracing::info!("Added `{appname}` with template: {template}, target: {target}.");
                apps.insert(
                    appname.to_string(),
                    AppEntry {
                        template: template.to_string(),
                        target: target.to_string(),
                    },
                );
            } else {
                tracing::error!("Use +app=template,target");
            }
        } else {
            tracing::error!("Use +app=template,target");
        }
    } else if let Some(appname) = op.strip_prefix('-') {
        if apps.remove(appname).is_none() {
            tracing::error!("No such app `{appname}`.");
        } else {
            tracing::info!("Removed {appname}.");
        }
    } else {
        tracing::error!("Unknown apps operation");
    }
}

pub fn pick_with_fzf(options: impl IntoIterator<Item = String>) -> io::Result<Option<String>> {
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
