# binfmt-dispatcher

binfmt-dispatcher is a simple dispatcher for [binfmt_misc](https://www.kernel.org/doc/html/latest/admin-guide/binfmt-misc.html) that dynamically picks the best interpreter to use at runtime.

## Usage
binfmt-dispatcher should be registered as a binfmt_misc handler. If using systemd, this is accomplished via [binfmt.d](https://www.freedesktop.org/software/systemd/man/latest/binfmt.d.html) (sample configs are provided for [x86](data/binfmt-dispatcher-x86.conf) and [x86-64](data/binfmt-dispatcher-x86-64.conf)). It is recommended to prefix the config with `zz-` or something akin to ensure it's parsed last, as systemd processes these in lexicographic order. After installing the config, remember to restart [systemd-binfmt.service](https://www.freedesktop.org/software/systemd/man/latest/systemd-binfmt.service.html) for it to take effect.

binfmt-dispatcher parses configuration from several sources:

- drop-in configs in `/usr/lib/binfmt-dispatcher.d/*.toml`
- `/etc/binfmt-dispatcher.toml`
- the running user XDG config (usually `$HOME/.config/binfmt-dispatcher/binfmt-dispatcher.toml`)

Configs are parsed in order and later settings win. A fully commented config is [provided](docs/binfmt-dispatcher.toml) and should be fairly self-explanatory.

When run as an interpreter, binfmt-dispatcher will parse the configs, pick the best interpreter to use based on it and the binary being run, and then run it. If enabled (via the `use_muvm` config setting), binfmt-dispatcher will use [muvm](https://github.com/AsahiLinux/muvm) to execute the interpreter in a microVM if the system page-size is not 4k.
