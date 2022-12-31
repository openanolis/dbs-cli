# DBS-CLI

> 如果需要更多关于命令行参数的信息: 请查阅 [`doc:args`](docs/args.md)
> 
> [[English Version]](README.md)

# 1. 简单示例:

如果想看到所有的参数：

```bash
./dbs-cli --help
```

一个简单例子：

```bash
./dbs-cli \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

如果你使用的kernel和rootfs来自firecraker,需要注意boot_args中的root参数:

```bash
./dbs-cli \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda" ;
```


如果需要显式地设置日志文件和日志等级,只需要添加`--log-file`和`--log-level`参数:

> `log-level`参数是大小写不敏感的: ErrOR 和 InFO 都是有效的log level.

```bash
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

创建一个使用socket通信的终端:

> 当参数`serial-path`没有指定值或者值为"stdio"时, `dbs-cli`将创建一个连接标准输入输出的终端.
> 
> 否则, `dbs-cli`将使用`serial-path`的值所指定的路径创建一个socket文件用以通信.

```
./dbs-cli \
  --log-file dbs-cli.log --log-level ERROR \
  --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/bionic.rootfs.ext4 \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" \
  --serial-path "/tmp/dbs" ;
```

# 2. 使用指南

## 1. 退出虚拟机

> 由于底层实现的限制, 目前不支持reboot操作, 如果需要退出虚拟机, 只需要在vm中输入`reboot`指令即可。

## 2. 开发者调试

如果想对该程序进行调试，而不想每次都编译一遍，可以通过以下方式执行该程序:

```bash
cargo run -- --kernel-path ~/path/to/kernel/vmlinux.bin \
  --rootfs ~/path/to/rootfs/rootfs.dmg \
  --boot-args "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1" ;
```

如果想看到帮助提示：

```bash
cargo run -- --help
```


## 3. 题外话

考虑到当前阶段上游依赖库存在一些版本冲突问题，如果你是第一次执行此程序，建议执行一下Makefile文件：

```bash
make build
```

如果期望使用自己的本地仓库取代现有的`dragonball`依赖的来源，可以参考[文档](docs/zh/dependency_zh.md)