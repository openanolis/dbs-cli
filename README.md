# DBS-CLI

> for more details about args: refer to [`doc:args`](docs/args.md)
> 
> [[简体中文版]](README_zh.md)

## Examples

See all options:

```
./dbs-cli --help
```

### Basic examples

```bash
./dbs-cli create \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" 
```

Set the log level and log file:

> The log-level argument is case-insensitive: ErrOR and InFO are valid.

```bash
./dbs-cli create \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1"
```

> tips: console=ttyS0 is used to connect to the guest console. If serial path is not defined, Dragonball will use stdio to interact with the guest. 
> If serial path is defined, nc -U /tmp/to/com1 could be used to connect the guest.
> console=ttyS1 and earlyprintk=ttyS1 are used to send guest dmesg to the log file, if not set, guest dmesg will not appear in log file.

Create a vsock console (communication with sock file)

> When the parameter `serial-path` is not given or set to "stdio", `dbs-cli` will create a stdio console.
> 
> Otherwise, `dbs-cli` will create a vsock console with a sock file, namely the value of `serial-path`.

```
./dbs-cli create \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --serial-path "/tmp/dbs"
```

Create a virtio-vsock tunnel for Guest-to-Host communication.

> When the parameter `vsock` is not given, `dbs-cli` will not add a virtio-vsock device.
> 
> Otherwise, `dbs-cli` will create a unix socket on the host using the argument
> specified with the `--vsock` parameter.

```
./dbs-cli create \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --vsock /tmp/vsock.sock
```

Create virtio-blk devices.

> The type of the `--virblks` receives an array of BlockDeviceConfigInfo in the
> format of JSON.

```
./dbs-cli create \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --virblks '[{"drive_id":"testblk","device_type":"RawBlock","path_on_host":"/path/to/test.img","is_root_device":false,"is_read_only":false,"is_direct":false,"no_drop":false,"num_queues":1,"queue_size":1024}]' 
```

### Networking

Start a Dragonball VMM with a virtio-based network device. `--virnets`
receives an array of `NetworkInterfaceConfig` in the format of JSON.

```
--virnets '[{"guest_mac":"43:2D:9C:13:71:48","backend":{"type":"vhost","iface_id":"eth0","host_dev_name":"tap0","allow_duplicate_mac":true}}]'
```

The supported network devices include:

```
// Virtio-net
{
	"guest_mac": "43:2D:9C:13:71:48",
	"backend": {
		"type": "virtio",
		"iface_id": "eth0",
		"host_dev_name": "tap0",
		"allow_duplicate_mac": true
	}
}

// Vhost-net
{
	"guest_mac": "43:2D:9C:13:71:48",
	"backend": {
		"type": "vhost",
		"iface_id": "eth0",
		"host_dev_name": "tap0",
		"allow_duplicate_mac": true
	}
}
```

### PCI Device

You can choose to attach a pci device during the boot time of Dragonball. 
Please note that hostdev_id and bus_slot_func are the must parameters for attaching pci device.  
Please make sure that pci device is binded to `vfio-pci` driver.

```
./dbs-cli create --kernel-path $KERNEL_PATH --rootfs $ROOTFS_PATH --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 root=/dev/vda1" --hostdev-id $HOST_DEVICE_ID --bus-slot-func $BUS_SLOT_FUNC
```

#### How to get hostdev_id and bus_slot_func?

hostdev_id: This is an id you pick for each pci device attaching into VM. So name it whatever number you want.

bus_slot_func: take a network device as the example, you could use `lspci | grep "network device"` and you could get something like
```
[root@xxx ~]# lspci | grep "network device"
5d:00.0 Ethernet controller: Red Hat, Inc. Virtio network device
5e:00.0 Ethernet controller: Red Hat, Inc. Virtio network device
```
`5d:00.0` is the bus_slot_func.

As an alternative way to insert a host device, you can use upcall to hotplug / hot-unplug a pci device into Dragonball while Dragonball is running, for more details please go to advanced usage part of this document.

## Advanced Usage

### Create API Server and Update VM

An API Server could be created by adding `--api-sock-path [socket path]`  into dbs-cli creation command.

After api socket created, you could use `./dbs-cli --api-sock-path [socket path] update` to send commands to the running VM.

Cpu Hotplug via API Server:

`sudo ./dbs-cli  --api-sock-path [socket path] update --vcpu-resize 2 `

Create hot-plug virtio-net devices via API Server:

**TODO: Needs to be updated**

> The type of the `--hotplug-virnets` receives an array of
> VirtioNetDeviceConfigInfo in the format of JSON.

```
sudo ./dbs-cli  \
  --api-sock-path [socket path] update \
  --hotplug-virnets "[{\"iface_id\":\"eth0\",\"host_dev_name\":\"tap0\",\"num_queues\":2, \"queue_size\":0,\"guest_mac\":\"43:2D:9C:13:71:48\",\"allow_duplicate_mac\":true}]" \
```

Create hot-plug virtio-blk devices via API Server:

> The type of the `--hotplug-virblks` receives an array of
> BlockDeviceConfigInfo in the format of JSON.

```
sudo ./dbs-cli  \
  --api-sock-path [socket path] update \
  --hotplug-virblks '[{"drive_id":"testblk","device_type":"RawBlock","path_on_host":"/path/to/test.img","is_root_device":false,"is_read_only":false,"is_direct":false,"no_drop":false,"num_queues":1,"queue_size":1024}]' \
```

Hotplug a pci device into Dragonball
```
./dbs-cli --api-sock-path $API_SOCK_PATH update --bus-slot-func $BUS_SLOT_FUNC --hostdev-id $HOST_DEVICE_ID
```

Prepare hot-unplug a pci device into Dragonball (must do before hotunplug)

```
./dbs-cli --api-sock-path ./sock update --prepare-remove-host-device $HOST_DEVICE_ID
```

Hot-unplug a pci device into Dragonball

```
./dbs-cli --api-sock-path ./sock update --remove-host-device $HOST_DEVICE_ID
```

TODO : add document for hot-plug virtio-fs

### Exit VM

If you want to exit vm, just input `reboot` in vm's console.

### For developers

If you wish to modify some details or debug to figure out the fault of codes, you can do as follow to see whether the program act expectedly or not.

```bash
cargo run -- --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 console=ttyS1 earlyprintk=ttyS1 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" 
```

To see some help:

```bash
cargo run -- --help
```

### Some off-topic remarks

Regarding the dependency issue of the upstream library, it is recommended to build the `build` target of Makefile to avoid it temporally.

```bash
make build
```

If the self-defined `dragonball` dependency is supposed to be used, please refer to [dependency document](docs/dependency.md)

## License

`DBS-CLI` is licensed under [Apache License](http://www.apache.org/licenses/LICENSE-2.0), Version 2.0.
