# pkgs

[English](./README.md) | [简体中文](./README.zh-CN.md)

Use symbolic links to manage the installation of different packages.

## Development background

### Managing configuration files

There has long been no unified solution for managing configuration files on Linux systems.
The mainstream solutions today are [Stow](https://www.gnu.org/software/stow/manual/stow.html) and [Nix Home Manager](https://github.com/nix-community/home-manager). However, both have some unsatisfactory drawbacks.

Stow’s management approach is somewhat rigid — it requires storing full filesystem paths and lacks customization options.
Nix Home Manager requires the Nix ecosystem and has a relatively high learning curve.

### Managing Git packages

> [!caution]
> This feature is still planned and has not yet been implemented.

Tools like [mpv](https://mpv.io/) lack suitable package management when installing plugins, forcing users to manage them manually, which is cumbersome.

This project also plans to integrate Git version management to provide a way to manage Git-based packages.

## Installation

```bash
cargo install pkgs-cli --locked
```

## Usage guide

Create a `pkgs.toml` configuration file in a directory that contains packages. The syntax is as follows:

```toml
# Optional `vars` section, used to define variables
# Use the ${var} syntax to reference variables
# If you reference other variables, they must be declared in order
[vars]
CONFIG_DIR = "${HOME}/.config" # HOME variable is built-in
APP_DIR = "${HOME}/Apps$"
YAZI_DIR = "${CONFIG_DIR}/yazi"
NU_DIR = "${CONFIG_DIR}/nushell"

# `packages` section is required; each table under it corresponds to a package,
# and should match a directory with the same name in the current directory
[packages.yazi]
type = "local" # Package type, optional; defaults to "local". Currently only "local" is supported.

[packages.yazi.maps] # Each entry under `maps` represents a mapping
"yazi.toml" = "${YAZI_DIR}/yazi.toml"         # Left side can be a file inside the package
"my-custom" = "${YAZI_DIR}/plugins/my-plugin" # It can also be a directory
"keymap.toml" = "${YAZI_DIR}/keymap.toml"     # Right side is the path where the symlink will be created
"yazi.nu" = "${NU_DIR}/autoload/yazi.nu"

[packages.nu.maps]
"config.nu" = "{NU_DIR}/config.nu"
```

The following commands are supported:

```bash
pkgs list # List all packages

pkgs load --all # Load all packages
pkgs load yazi nu # Load only yazi and nu
# After `load`, if you modify the configuration file you can run `load` again to reapply; `unload` is not required

pkgs unload --all # Unload all packages
pkgs unload yazi nu # Unload only yazi and nu
```

### Behavior

The `load` command creates symbolic links at the specified locations that point to the corresponding files' **absolute paths** according to the configuration file (so if a file path changes because of variables, you must `load` again). If an error occurs while loading a package, the operation for that package will be **rolled back**. After loading completes, the created symlinks are recorded in `.pkgs/trace.toml` in the current directory — please **do not** modify or delete this file.

If a parent directory for a target path does not exist during loading, the tool will **create all missing parent directories** and notify the user.

The `unload` command removes packages by reading `.pkgs/trace.toml`. If an error occurs during unload, a **rollback** will also be performed.

## License

This project is open-source under the [GPL-3.0 License](https://www.gnu.org/licenses/gpl-3.0.en.html). See [LICENSE](./LICENSE) for details.
