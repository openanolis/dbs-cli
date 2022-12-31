# 这篇文档描述了如何使用自定义的Dragonball依赖

[English version](../dependency.md)

目前，`dragonball`依赖的来源是`kata-containers`仓库，但是它的来源可以被替换。

```toml
[dependencies]
dragonball = { git = "https://github.com/kata-containers/kata-containers", branch = "main" }
hypervisor = { git = "https://github.com/kata-containers/kata-containers", branch = "main" }
```

有一个代码合并请求 [`dragonball: Fix problem that stdio console cannot connect to stdout`](https://github.com/kata-containers/kata-containers/pull/5082)解决了一个BUG，从而使得`dbs-cli`可以正常运行。因此，在此PR之后从`kata-containers`分叉出的仓库都能替换原有的依赖源，并且允许有额外的代码修改。