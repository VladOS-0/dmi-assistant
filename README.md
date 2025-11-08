# DMI Assistant

Simple application for finding and viewing DMI icon files from [BYOND](https://www.byond.com/) game engine.

## Features
 * **DMI Explorer:** recursively find DMI's in folders with searching by file names and icon state names.
 * **DMI Viewer:** View DMI icons with resizing, animations, copying as GIFs and searching by icon state names.

## Installation
 * **Download from GitHub Releases** - go to the [latest release](https://github.com/VladOS-0/DMIAssistant/releases/), grab archive for your OS, unpack it somewhere, change `.env` file if needed, [customize and place in the right place](#Customization) `Config.toml` if you want.

 * **Build from source** - clone this repository, [install Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and build the DMIAssistant with `cargo build --release`. Transfer your executable from `./target/release` to the another place if you want, optionally make `.env` file using [.env.example](/.env.example) as an... well, example. Rename `Config.example.toml` to `Config.toml`, then [customize and place in the right place](#Customization) if you want.

## Customization
 All settings are stored in the `Config.toml` file, which is generated automatically by the application at the first launch. It is placed in the config directory, which is *probably*:
 * `/home/user/.config/DMIAssistant` on **GNU/Linux**
 * `/Users/User/Library/Application Support/com.Vlad0s.DMIAssistant` on **MacOS**
 * `C:\Users\User\AppData\Local\Vlad0s\DMIAssistant\config` on **Microsoft Windows**

 ...where `user`/`User` part is your username. Also path to the config directory can be overriden in `.env` file with the `CONFIG_PATH` variable.
 
 > [!ATTENTION]  
 > Be **VERY** careful, when you modify the paths in the config, **especially** `log_dir` and `cache_dir`.
 > These directories will be **NUKED** by the DMIAssistant, deleting all your important files, if you set them wrongly!

 Visit [Config.example.toml](/Config.example.toml) to see, what exactly you can customize.

## License
 Copyright (C) 2025 Vlad0s <vladstolyarchuk7@gmail.com>

 Licensed under the GNU General Public License 3 or later. For more information see the [LICENSE](./LICENSE).