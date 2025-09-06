# pkgs

[English](./README.md) | [简体中文](./README.zh-CN.md)

使用软链接来管理不同包的安装。

## 开发背景

### 管理配置文件

Linux 系统下配置文件的管理一直以来都没有统一的解决方案。
目前主流的方案主要有 [Stow](https://www.gnu.org/software/stow/manual/stow.html) 和 [Nix Home Manager](https://github.com/nix-community/home-manager)。
但它们都存在一些令人不太满意的缺点。

Stow 的管理方式比较死板，需要保存完整的文件系统路径，且缺乏可定制项。
而 Nix Home Manager 需要 Nix 生态，上手门槛较高。

### 管理 Git 包

> [!caution]
> 该功能仍在计划中，尚未实现。

像 [mpv](https://mpv.io/) 等工具在安装插件时也缺乏合适的管理工具，只能由用户手动管理，相当麻烦。

本插件也打算集成 Git 版本管理，提供一种管理 Git 包的手段。

## 软件安装

```bash
cargo install pkgs-cli --locked
```

## 使用指南

在存在包的目录下创建描述文件，支持 TOML 和 YAML 两种格式，如下所示：

<details>
<summary>pkgs.toml</summary>

```toml
# vars 字段，可选，用于配置变量
# 使用 ${var} 语法以调用变量
# 如要引用其他变量，必须按顺序声明
[vars]
CONFIG_DIR = "${HOME}/.config" # HOME 变量已内置
APP_DIR = "${HOME}/Apps"
NU_DIR = "${CONFIG_DIR}/nushell"

# packages 字段，必选，其下每个表对应一个包，对应当前目录下与包同名的目录
[packages.yazi]
type = "local" # 包类型，可选，默认为 local，当前仅支持 local

[packages.yazi.vars] # 包局部变量，仅在包内部可见
YAZI_DIR = "${CONFIG_DIR}/yazi"

[packages.yazi.maps] # maps 下的每个关系对应一个映射
"yazi.toml" = "${YAZI_DIR}/yazi.toml"         # maps 左边可以是包下面的一个文件
"my-custom" = "${YAZI_DIR}/plugins/my-plugin" # 也可以是一个文件夹
"keymap.toml" = "${YAZI_DIR}/keymap.toml"     # 右边则是对应要创建的软链接

"yazi.nu" = "${NU_DIR}/autoload/"             # 若映射文件同名，可直接以 / 结尾，省略文件名

[packages.nu.maps]
"config.nu" = "${NU_DIR}/"
```

</details>

<details>
<summary>pkgs.yaml / pkgs.yml</summary>

```yaml
# vars 字段，可选，用于配置变量
# 使用 ${var} 语法以调用变量
# 如要引用其他变量，必须按顺序声明
vars:
  CONFIG_DIR: ${HOME}/.config # HOME 变量已内置
  APP_DIR: ${HOME}/Apps
  NU_DIR: ${CONFIG_DIR}/nushell

# packages 字段，必选，其下每个表对应一个包，对应当前目录下与包同名的目录
packages:
  yazi:
    type: local # 包类型，可选，默认为 local，当前仅支持 local

    vars: # 包局部变量，仅在包内部可见
      YAZI_DIR: ${CONFIG_DIR}/yazi

    maps: # maps 下的每个关系对应一个映射
      yazi.toml: ${YAZI_DIR}/yazi.toml         # maps 左边可以是包下面的一个文件
      my-custom: ${YAZI_DIR}/plugins/my-plugin # 也可以是一个文件夹
      keymap.toml: ${YAZI_DIR}/keymap.toml     # 右边则是对应要创建的软链接

      yazi.nu: ${NU_DIR}/autoload/             # 若映射文件同名，可直接以 / 结尾，省略文件名

  nu:
    maps:
      config.nu: ${NU_DIR}/
```

</details>

支持以下命令：

```bash
pkgs list # 列出所有包

pkgs load --all # 加载所有包
pkgs load yazi nu # 仅加载 yazi 与 nu
# load 加载后如果修改配置文件，可以再次运行 load 重新应用，不必 unload

pkgs unload --all # 卸载所有包
pkgs unload yazi nu # 仅卸载 yazi 与 nu
```

### 行为

`load` 命令会根据配置文件中的描述，在指定位置创建指向相应文件**绝对路径**的软链接（因此如果文件路径发生变量，需要重新 `load`）。
如果在加载某个包的过程中发生错误，会**回滚**本次对该包的操作。
加载完成后在会在当前目录的 `.pkgs/trace.toml` 下记录所创建的软链接，请**不要**修改或删除这个文件。

如果加载时某个路径对应的父文件夹不存在，当前会**直接创建所有父文件夹**，并提示用户。

`unload` 命令则是通过读取 `.pkgs/trace.toml` 来卸载相应的包。当卸载出错时，也会进行**回滚**操作。

## 许可协议

项目采用 [GPL-3.0 协议](https://www.gnu.org/licenses/gpl-3.0.en.html)开源，详见 [LICENSE](./LICENSE)。
