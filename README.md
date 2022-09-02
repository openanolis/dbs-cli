# DBS-CLI

> for more details about args: refer to [`doc:args`](docs/args.md)
> 
> [[简体中文版]](README_zh.md)

# 1. Examples:

See all options:

```
./dbs-cli --help
```

A simple example:

```bash
./dbs-cli \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

For the rootfs from firecracker:

```bash
./dbs-cli \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda" ;
```


Set the log level and log file:

> The log-level argument is case-insensitive: ErrOR and InFO are valid.

```bash
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

Create a vsock console (communication with sock file)

> When the parameter `serial-path` is not given or set to "stdio", `dbs-cli` will create a stdio console.
> 
> Otherwise, `dbs-cli` will create a vsock console with a sock file, namely the value of `serial-path`.

```
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --serial-path "/tmp/dbs" ;
```

# 2. Usage

## 1. Exit vm

> If you want to exit vm, just input `reboot` in vm's console.

## 2. For developers

If you wish to modify some details or debug to figure out the fault of codes, you can do as follow to see whether the program act expectedly or not.

```bash
cargo run -- --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

To see some help:

```bash
cargo run -- --help
```

## 3. Some off-topic remarks

Regarding the dependency issue of the upstream library, it is recommended to build the `build` target of Makefile to avoid it temporally.

```bash
make build
```

# License

`DBS-CLI` is licensed under [Apache License](http://www.apache.org/licenses/LICENSE-2.0), Version 2.0.