use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "colorctl",
    version,
    about = "Tiny templater with vars + app targets"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Apply all app templates using current variables
    Apply,
    /// List variables or modify with +name=value / -name
    Vars {
        /// Example: +num=4 or -color
        #[arg(value_name = "OP", allow_hyphen_values = true)]
        op: Vec<String>,
    },
    /// List apps or modify with +app=template,target or -app
    Apps {
        /// Example: +app1=~/.t.tmpl,~/.t or -app2
        #[arg(value_name = "OP", allow_hyphen_values = true)]
        op: Vec<String>,
    },
    /// Edit app templates in $EDITOR. No args => pick via fzf.
    Edit {
        /// App names to edit (templates)
        apps: Vec<String>,
    },
}
