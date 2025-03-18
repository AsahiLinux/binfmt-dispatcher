# binfmt-dispatcher

binfmt-dispatcher is a simple dispatcher for [binfmt_misc](https://www.kernel.org/doc/html/latest/admin-guide/binfmt-misc.html) that dynamically picks the best interpreter to use at runtime.

## Installation
binfmt-dispatcher is packaged in Fedora as of Fedora Linux 40. It can be installed with:

```
$ sudo dnf install binfmt-dispatcher
```

which will automatically register it as a binfmt_misc handler via systemd and installer a basic configuration.

## Configuration
binfmt-dispatcher should be registered as a binfmt_misc handler. If using systemd, this is accomplished via [binfmt.d](https://www.freedesktop.org/software/systemd/man/latest/binfmt.d.html) (sample configs are provided for [x86](data/binfmt-dispatcher-x86.conf) and [x86-64](data/binfmt-dispatcher-x86-64.conf)). It is recommended to prefix the config with `zz-` or something akin to ensure it's parsed last, as systemd processes these in lexicographic order. After installing the config, remember to restart [systemd-binfmt.service](https://www.freedesktop.org/software/systemd/man/latest/systemd-binfmt.service.html) for it to take effect.

binfmt-dispatcher parses configuration from several sources:

- drop-in configs in `/usr/lib/binfmt-dispatcher.d/*.toml`
- `/etc/binfmt-dispatcher.toml`
- the running user XDG config (usually `$HOME/.config/binfmt-dispatcher/binfmt-dispatcher.toml`)

Configs are parsed in order and later settings win. A fully commented config is [provided](docs/binfmt-dispatcher.toml.example) and should be fairly self-explanatory.

## Usage
When run as an interpreter, binfmt-dispatcher will parse the configs, pick the best interpreter to use based on it and the binary being run, and then run it. If enabled (via the `use_muvm` config setting), binfmt-dispatcher will use [muvm](https://github.com/AsahiLinux/muvm) to execute the interpreter in a microVM if the system page-size is not 4k. If the interpreter or any of its dependencies are missing, binfmt-dispatcher will attempt to install them by invoking the package manager.

If run outside of a terminal session, binfmt-dispatcher will assume it's being run as part of a desktop environment (e.g. because a user double clicked on an x86-64 binary in a file manager). If [zenity](https://help.gnome.org/users/zenity/stable/) is installed, dialog boxes will be displayed to provide feedback to the user when necessary. If it's necessary to install any missing dependencies, the package manager will be run inside a terminal emulator via [xdg-terminal-exec](https://github.com/Vladimir-csp/xdg-terminal-exec).

## License
This project is [MIT](https://spdx.org/licenses/MIT.html) licensed. See the [LICENSE](LICENSE) file for the full text of the license.
