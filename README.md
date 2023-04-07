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
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" create ;
```

For the rootfs from firecracker:

```bash
./dbs-cli \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda" create ;
```


Set the log level and log file:

> The log-level argument is case-insensitive: ErrOR and InFO are valid.

```bash
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" create ;
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
  --serial-path "/tmp/dbs" creare;
```

Create a virtio-vsock tunnel for Guest-to-Host communication.

> When the parameter `vsock` is not given, `dbs-cli` will not add a virtio-vsock device.
> 
> Otherwise, `dbs-cli` will create a unix socket on the host using the argument
> specified with the `--vsock` parameter.

```
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --vsock /tmp/vsock.sock create;
```

Create virtio-net devices.

> The type of the `--virnets` receives an array of VirtioNetDeviceConfigInfo in the
> format of JSON.

```
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --virnets "[{\"iface_id\":\"eth0\",\"host_dev_name\":\"tap0\",\"num_queues\":2,\"queue_size\":0,\"guest_mac\":\"43:2D:9C:13:71:48\",\"allow_duplicate_mac\":true}]" \
  create;
```

# 2. Usage

## 1. Create API Server and Update VM

An API Server could be created by adding `--api-sock-path [socket path]`  into dbs-cli creation command.

After api socket created, you could use `./dbs-cli --api-sock-path [socket path] update` to send commands to the running VM.

Right now, we have only one command for cpu resizing, and here is the command example.

`sudo ./dbs-cli  --api-sock-path [socket path] --vcpu-resize 2 update`

Create hot-plug virtio-net devices via API Server.

> The type of the `--hotplug-virnets` receives an array of
> VirtioNetDeviceConfigInfo in the format of JSON.

```
sudo ./dbs-cli  \
  --api-sock-path [socket path]
  --hotplug-virnets "[{\"iface_id\":\"eth0\",\"host_dev_name\":\"tap0\",\"num_queues\":2,\"queue_size\":0,\"guest_mac\":\"43:2D:9C:13:71:48\",\"allow_duplicate_mac\":true}]" \
  update
```

## 2. Exit vm

> If you want to exit vm, just input `reboot` in vm's console.

## 3. For developers

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

If the self-defined `dragonball` dependency is supposed to be used, please refer to [dependency document](docs/dependency.md)

# License

`DBS-CLI` is licensed under [Apache License](http://www.apache.org/licenses/LICENSE-2.0), Version 2.0.
