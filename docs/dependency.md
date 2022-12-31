# This document demonstrate how to connect to user-defined Dragonball crate

[中文简体](zh/dependency_zh.md)

The dependency `dragonball` is currently relied on a remote repository, while the source can be replaced by user-defined location.

```toml
[dependencies]
dragonball = { git = "https://github.com/kata-containers/kata-containers", branch = "main" }
hypervisor = { git = "https://github.com/kata-containers/kata-containers", branch = "main" }
```

The pull request [`dragonball: Fix problem that stdio console cannot connect to stdout`](https://github.com/kata-containers/kata-containers/pull/5082) has fixed a bug, which enables this crate to work well. Thus any repo forked from `kata-containers` with the bug mentioned above fixed can be directly employed, along with additional improvements permitted.