# keyring · 系统钥匙串里的密钥

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-KEYRING<br>
> 对象：[`keyring` 4.2.0](https://crates.io/crates/keyring/4.2.0)（2026-08-29 发布）＝ [`keyring-core` 1.0.0](https://crates.io/crates/keyring-core/1.0.0) + 各平台 store crate（下表）<br>
> 许可证：keyring、keyring-core 及 open-source-cooperative 维护的全部 store crate 均为 MIT OR Apache-2.0

## 定位

我们用它保管 control 持有的密钥（provider 令牌、Matrix 应用服务令牌等），落实[系统边界 §控制面自己的存储](../../design/spec/system.md#控制面自己的存储)「密钥不进账本、不进 Git、不进 Room、不进 Context」这条约束：桌面机上放进 OS 钥匙串——macOS Keychain、Windows Credential Manager、Linux Secret Service。它替代的手写代码是三平台各一套 FFI 或调 `security` / `secret-tool` 命令行的胶水。

## 上游能力

- **架构（v4）**：API 在 `keyring-core` 1.0.0（`Entry::new / set_password / get_password / set_secret / get_secret / delete_credential / get_attributes`，`set_default_store` 装入一个 store），存储后端拆成独立 crate。`keyring` 4.2.0 只是把三个默认 store 打包成 `v1` feature，保持旧 API 不变；上游明确建议正式应用直接依赖 `keyring-core` + 所选 store，不要用 `cli` feature（它会拉进全部 store）。
- **store crate 一览**（crates.io，2026-09-03）：

| store crate | 版本 / 更新 | 平台 | 后端 | 在 `keyring` 4.2.0 中 |
| --- | --- | --- | --- | --- |
| apple-native-keyring-store | 1.0.2 / 2026-08-06 | macOS、iOS | 传统 Keychain（`keychain` feature）或 Data Protection（`protected`） | 默认 `v1` |
| windows-native-keyring-store | 1.1.0 / 2026-05-24 | Windows | Credential Manager 通用凭据，默认 Enterprise 持久级别 | 默认 `v1` |
| zbus-secret-service-keyring-store | 1.0.1 / 2026-08-15 | Linux、BSD | Secret Service（D-Bus，zbus 实现） | 默认 `v1` |
| dbus-secret-service-keyring-store | 1.0.1 / 2026-08-15 | Linux、BSD | Secret Service（libdbus 实现） | 可选 |
| linux-keyutils-keyring-store | 1.0.0 / 2026-04-21 | Linux | 内核 keyctl，用户持久 keyring + 会话 keyring | 可选 |
| db-keystore | 0.5.2 / 2026-08-26 | 64 位桌面 | 加密 SQLite 文件（turso 实现），第三方维护，3 star，钉 turso =0.7.2 | 可选 |
| android-native-keyring-store | 1.x | Android | — | 可选 |

- **MSRV**：keyring 4.2.0 与 zbus/windows store 要 1.88.0，keyring-core 1.85；HCTL2 的 1.98.0 满足。
- **依赖树**（`cargo tree`，rustc 1.98.0）：`keyring` 默认 `v1` 在 Linux 拉 100 个 crate（zbus 栈），macOS 10 个，Windows 13 个；`keyring-core` + `linux-keyutils-keyring-store` 只有 7 个。
- 维护：keyring 4.2.0 下载 2,318 万，近 90 天 957 万；仓库 765 star，0 个 open issue，最后提交 2026-08-29。4.0.0 于 2026-04-26 发布，之后四个月内出了 4.0.1 到 4.2.0 共十个版本，处于活跃迭代期。
- **错误模型**：`Error::{NoEntry, Ambiguous, NoStorageAccess, PlatformFailure, Invalid, TooLong}`；`keyring-core` 附带 mock store 供测试，sample store 明确声明不保证安全。

## 候选比较

| 候选 | 版本 / 日期 | 许可证 | 覆盖 | 依赖树 | 备注 |
| --- | --- | --- | --- | --- | --- |
| keyring 4.2.0（`v1`） | 2026-08-29 | MIT OR Apache-2.0 | macOS / Windows / Linux 桌面 | 10 / 13 / 100 | 最短路径，旧 API 不变 |
| keyring-core + 逐平台 store | 1.0.0 + 各 1.x | MIT OR Apache-2.0 | 同上，可加 keyutils | 按需 | 上游推荐的正式用法 |
| 直接调 `security` / `secret-tool` / `cmdkey` | OS 自带 | — | 三平台各一套 | 0 | 要解析命令行输出，桌面会话依赖相同 |
| 自写 FFI（security-framework、zbus、windows-sys） | — | MIT/Apache | 同上 | 与 keyring 相当 | 等于重写 keyring 的 store 层 |
| 加密文件（age / 自管密钥） | — | — | 全平台含 headless | 小 | 密钥从哪来仍是问题 |

## 边界与取舍

- **Linux 桌面**：Secret Service 需要一个用户会话 D-Bus 和正在运行、已解锁的 Secret Service 守护进程（gnome-keyring、KWallet 等）。没有这两样时默认 store 报 `NoStorageAccess` 或 `PlatformFailure`。
- **无桌面会话的 Linux 服务器（headless 回退）**：linux-keyutils store 的文档直说「在 headless 或没有 gnome-keyring 的机器上强烈建议用这个 store」，但同一份文档也写明内核密钥设施「完全在内存里，不跨重启持久」——登出后经用户持久 keyring 还能找回，重启就没了。所以 keyutils 只能做**会话缓存**，不能做服务器的持久密钥来源。持久来源三选一，都在 keyring 之外：
  1. 用 `dbus-run-session` 拉一个无桌面的 Secret Service 守护进程并在启动时解锁——多一个常驻进程、多一个解锁口令，运维成本高，本轮未验证；
  2. db-keystore 之类加密文件 store——加密密钥本身又要放在某处，且该 crate 第三方维护、钉死 turso 0.7.2，引入分量不小；
  3. 交给宿主机制：systemd 的凭据机制（`LoadCredential` / `LoadCredentialEncrypted`）或权限 0600 的文件，由部署者负责——这是服务器场景的业界常规做法。
  建议把「密钥提供者」做成 control 内部的一个薄接口：桌面走 keyring，headless 走文件/systemd 凭据并在 `doctor` 里明示当前来源；keyutils 作为进程重启间的缓存，不作为权威。
- **macOS**：传统 Keychain store 对所有应用可用；Data Protection store 只对沙盒应用可用（macOS 10.15+），部分能力还要 provisioning profile——HCTL2 的命令行 control 用前者。钥匙串锁定（例如 ssh 登录且未解锁 login keychain）与首次访问的授权弹窗行为，上游 1.0.2 文档摘录未写明，须在 macOS CI 上实测后再写进契约测试。
- **Windows**：Credential Manager 通用凭据，`target_name` = `user` + `.` + `service`（默认分隔符），默认 Enterprise 持久级别；单条凭据大小上限在 store 文档里未查到，微软 API 有上限（未在本轮核对具体字节数），大密钥应存引用而不是内容。
- **许可证**：主链 MIT OR Apache-2.0；Linux 侧 zbus 栈（MIT）体量大但许可无碍。db-keystore 声明 MIT or Apache-2.0，其依赖 turso 为 MIT。
- **线程**：keyring-core 自身线程安全，但文档提醒底层 store 对同一凭据的多线程访问不一定可靠——control 是单写者，把密钥读写收敛到一个位置即可。

## 决定建议

**采用 SDK**：按上游建议依赖 `keyring-core` 1.0.0 加逐平台 store（apple-native `keychain`、windows-native、zbus-secret-service），不用 `keyring` 的 `cli` feature；短期用 `keyring` 4.2.0 的 `v1` feature 起步也可以，两者 API 相同。headless Linux 不由 keyring 独自解决：keyutils 作会话缓存，持久来源交给 systemd 凭据或 0600 文件，`doctor` 报告来源——这一条涉及部署姿态，需要所有者拍板。

## 证据

- crates.io：[keyring 4.2.0](https://crates.io/crates/keyring/4.2.0)、[keyring-core 1.0.0](https://crates.io/crates/keyring-core/1.0.0)、[apple-native-keyring-store 1.0.2](https://crates.io/crates/apple-native-keyring-store/1.0.2)、[windows-native-keyring-store 1.1.0](https://crates.io/crates/windows-native-keyring-store/1.1.0)、[zbus-secret-service-keyring-store 1.0.1](https://crates.io/crates/zbus-secret-service-keyring-store/1.0.1)、[dbus-secret-service-keyring-store 1.0.1](https://crates.io/crates/dbus-secret-service-keyring-store/1.0.1)、[linux-keyutils-keyring-store 1.0.0](https://crates.io/crates/linux-keyutils-keyring-store/1.0.0)、[db-keystore 0.5.2](https://crates.io/crates/db-keystore/0.5.2)
- 文档：[keyring 4.2.0 crate 文档](https://docs.rs/keyring/4.2.0/keyring/)、[keyring-core 1.0.0 文档](https://docs.rs/keyring-core/1.0.0/keyring_core/)、[linux-keyutils store 文档（headless 建议与不跨重启说明）](https://docs.rs/linux-keyutils-keyring-store/1.0.0/linux_keyutils_keyring_store/)、[apple-native store 文档](https://docs.rs/apple-native-keyring-store/1.0.2/apple_native_keyring_store/)、[windows-native store 文档](https://docs.rs/windows-native-keyring-store/1.1.0/windows_native_keyring_store/)
- 仓库：[keyring-rs README 与 Cargo.toml（v4 架构、feature 定义）](https://github.com/open-source-cooperative/keyring-rs)、[Keyring 生态 wiki](https://github.com/open-source-cooperative/keyring-rs/wiki/Keyring)、[db-keystore README](https://github.com/stevelr/db-keystore)
- 规范：[freedesktop Secret Service](https://specifications.freedesktop.org/secret-service/latest/description.html)、[systemd 凭据](https://systemd.io/CREDENTIALS/)
- 本库：[系统边界 §控制面自己的存储](../../design/spec/system.md#控制面自己的存储)、[部件矩阵表 D Secret store 一行](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)
