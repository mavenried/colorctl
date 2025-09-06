use crate::utility::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};
pub fn vars_ctl(op: &str, vars: &mut Vars) {
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

pub fn apps_ctl(op: &str, apps: &mut Apps) {
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
