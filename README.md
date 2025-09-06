<h1 align="center"> colorctl </h1>

<p align="center" >
 A simple configuration templating tool written in Rust. It allows you to define variables and application templates, then apply them to generate real configuration files.
</p>

## Installation

Build from source with Cargo:

``` bash
cargo install --path .
```

## Usage

``` bash
colorctl [COMMAND] [ARGS]
```
It finds variables defined in `$[]` symbols to substitute
### Commands

-   `apply`\
    Apply all templates with the current variables.

-   `vars`\
    Manage variables.

    -   `colorctl vars` → List variables\
    -   `colorctl vars +name=value` → Add or update variable\
    -   `colorctl vars -name` → Remove variable

-   `apps`\
    Manage applications (template + target pairs).

    -   `colorctl apps` → List applications\
    -   `colorctl apps +appname=template_path,target_path` → Add app\
    -   `colorctl apps -appname` → Remove app

-   `edit`\
    Open an app's template in your `$EDITOR`.

    -   `colorctl edit` → Pick an app with `fzf` and edit its template\
    -   `colorctl edit app1 app2` → Edit specific apps

-   `help`\
    Show usage information.

## Example

``` bash
# Add a variable
colorctl vars +color=blue

# Add an app (template, target)
colorctl apps +nvim=~/Templates/colorctl/init.vim,~/.config/nvim/init.vim

# Apply all configs
colorctl apply
```

This will substitute variables like `$[color]` inside your template
files and write the processed output to the target files.


