# Buck2 替换方案：第一方构建图与外部子系统边界

> 状态：已落地 · 第一方、外部组包、完整发行与测试均进入 Buck action graph<br>
> 基线：main @ 376cb7f；实现从 PR #15 起逐批合入<br>
> 去向：已进入 src/ 第一方构建、CI 与发行组装；后续按显式工具链升级维护<br>
> 日期：2026-08-27  
> 说明：Accepted · 作为后续四批实现 PR 的施工依据  
> 关联：[Grok 构建确定性备忘](./grok-20260827a.md)  
> 范围：HCTL2 第一方代码、工具链、CI/CD 与最终发行组装；不重写外部独立产品的原生构建图。

## 决定

HCTL2 采用 Buck2 管理自己的跨平台、多语言编译环境。选择理由不是替代 Cargo、pnpm、Vite 或上游项目的原生构建系统，而是把第一方源码、工具链、平台、生成动作、测试与产物纳入同一张可查询、可缓存、可验证的依赖图。

外部独立产品不做细粒度 Buck2 化，但它们的粗粒度构建边界必须进入 HCTL2 的 Buck action graph。Tuwunel、Vikunja、Dagu、tmux、Cinny 与 Static Web Server 继续按各自上游形态获取或编译：有官方二进制时由 Buck 原生下载规则消费固定版本与摘要的官方制品；需要源码编译时由一个外部 action 调用上游 Cargo、`configure && make` 等原生入口。HCTL2 不维护其内部 crate、Go package 或 C target 图。

这条边界也修正了“所有我们交付的组件必须使用同一编译器版本”的过宽表述：HCTL2 自己编译的同一种语言默认共用一把钉死的工具链；外部子系统使用其上游工具链，版本与构建环境进入该子系统自己的记录。官方二进制视为摘要锁定的外部 blob，除非上游提供可验证 provenance，不推断它使用了哪一把编译器。

## 当前基线

第一方代码当前是 Rust 1.98.0 Cargo workspace，包含 `hctl2-agentd`、`hctl2-tool` 与 `hctl2-protocol`。Ubuntu 上复跑现有 CI 路径，格式、Clippy、workspace build 与 11 个测试全部通过。

`src/packaging/dependencies` 约有 2,500 行 shell。它已经固定远端 URL、版本、源码 commit 与 SHA-256，生成运行包、源码伴随包、许可证和 payload manifest，也具备安装、启停和 smoke 测试；但下载、编译、组包与宿主机工具链仍由 shell 顺序和人工 cache marker 串联。GitHub Actions 当前只检查 Rust workspace 和 shell 语法，没有构建外部子系统包，没有 Release workflow、签名、notarization、artifact provenance 或受保护的 `main` 必需检查。

## 四类构建对象

| 类型 | 例子 | 归属 |
| --- | --- | --- |
| HCTL2 第一方代码 | agentd、tool、protocol、未来 control、CLI | Buck2 原生目标 |
| 编进第一方产物的库 | Rust crates、未来 npm packages | Cargo/Reindeer 或 pnpm lock 是事实源；进入第一方 Buck 图 |
| 外部独立子系统 | Tuwunel、Vikunja、Dagu、tmux、Cinny、Static Web Server | 保持上游原生构建；以粗粒度产物合同接入 |
| 运行期资产 | config、安装与 `start/stop/status/smoke` 脚本、静态内容 | 作为声明输入被组包和测试，不误写成编译规则 |

这里的“第三方库”和“外部子系统”不能混为一谈。链接进 HCTL2 二进制的 crate 或 npm package 属于第一方编译闭包，必须受 HCTL2 工具链和锁文件约束；拥有独立进程、发布物与原生构建系统的上游产品是外部子系统，Buck2 不进入其内部图。

## 目标拓扑

```text
HCTL2 source
    │
    ├── pinned Rust / Node / pnpm / Tauri toolchains
    ├── code generation, lint and tests
    └── Buck2 first-party artifacts
                         │
                         ▼
                  release assembly
                         ▲
                         │
external subsystem builds
    ├── chatroom package
    ├── kanban package
    ├── workflow package
    └── terminal package
```

这里的连线是 Buck target 的真实依赖边，不是 workflow 中约定的执行顺序。发行输入目标同时依赖第一方导出和外部子系统目标；外部目标的命令、环境、声明输入、工具链和平台共同形成 Buck action key。供应链 SHA-256 是下载规则的输入校验，不再额外发明一套“package fingerprint”。

外部子系统向发行层只暴露固定合同：

- 目标平台身份；
- 运行归档与 SHA-256 sidecar；
- 许可证要求所需的源码伴随归档；
- dependencies、sources 与 licenses manifest；
- 可离线执行的安装和生命周期测试结果。

最终发行组装可以把这些归档作为不可透明修改的输入，但不得借此把上游内部构建逻辑复制进 HCTL2 Buck rules。

## 平台命名

不把 `macos` 全局改写成 `darwin`。业界成熟做法是按语义分层：产品和 Buck2 OS constraint 使用 `macos`，Rust target triple、Go/Node 平台与 `uname` 边界按各生态使用 `darwin`。

第一批 Buck platform 使用：

- `linux_x86_64_gnu`；
- `macos_x86_64`；
- `macos_arm64`。

平台定义分别记录 Buck OS/CPU constraint、Rust target triple、宿主检测值和上游 asset aliases。例如 `macos_arm64` 映射到 Rust 的 `aarch64-apple-darwin`、Go 的 `darwin/arm64` 与 `uname` 的 `Darwin/arm64`。目录 `platforms/macos` 保留产品平台语义，不使用混合生态字符串承担多种身份。

## 工具链纪律

- Buck2 本身使用仓库内薄 launcher 与 DotSlash binary manifest，钉定日期发行版及匹配的 Prelude commit，不使用移动的 `latest`；本机 REAPI cache server 也以 DotSlash 钉定官方单二进制和摘要。
- HCTL2 第一方 Rust target 共用 Rust 1.98.0；升级是一项显式、全平台回归的变更。
- Cargo manifests 和 lockfile 继续是 Rust 依赖事实源；Reindeer 生成第三方 crate 的 Buck targets，不手写两份依赖图。
- Workbench 落地后，pnpm lock 与 Cargo lock 继续各自拥有语言依赖事实；Buck2 钉 Node、pnpm、Rust、Tauri CLI，编排 Vite 与 Tauri2 的原生构建入口。第一版不使用 Buck2 JS Prelude 逐模块替代 pnpm/Vite。
- macOS 的 SDK、deployment target、架构、linker 与签名身份是 target-specific 输入。Linux 与 Apple Clang 不因版本字符串不同而被伪装成同一发行物。
- Buck2 本地 action 目前并不自动获得严格沙箱；“未声明输入不可见”的硬保证需要 Remote Execution 或受控构建环境。当前只接入 cache、未接入远端执行，因此文档只承诺工具链钉定、动作输入身份和缓存失效，不夸大本地 hermeticity。

## 仓库形状

Buck workspace 严格位于 `src/`，仓库根不放置 `.buckconfig`、`.buckroot` 或 `BUCK`。发行 action 使用 `packaging/release/assets/` 中的许可证和用户文档快照；仓库级 `LICENSE` 与 `docs/usage.md` 继续作为文档事实源，由 CI 机械校验快照一致性。这样既保持 `root//...` 标签和从 `src/` 调用 `./buck2` 的入口，也不把构建系统边界扩大到整个仓库：

```text
repository/
├── LICENSE                 # 仓库文档事实源
├── docs/usage.md           # 仓库文档事实源
└── src/                    # Buck workspace / root cell
    ├── .buckconfig
    ├── .buckroot
    ├── buck2
    ├── build/{platforms,toolchains,rules}/
    ├── apps/*/BUCK
    ├── crates/*/BUCK
    ├── third-party/rust/
    └── packaging/
        └── release/assets/ # 进入发行 action 的受检快照
```

目录随首个真实使用场景创建，不提前铺 Workbench、Node 或未来语言的空壳。

## 四批实现

### PR 1 · Buck2 基础设施

- 钉 Buck2、DotSlash 与匹配 Prelude；
- 建立三个 platform configuration；
- 建立 Rust 1.98.0 toolchain；
- 提供在 `src/` 运行的 `./buck2 build root//...` 与 `./buck2 test root//...` 稳定入口；
- 验证工具链或平台定义变化会让相关 action 失效。

### PR 2 · 第一方 Rust workspace

- 为 protocol、agentd 与 tool 建立 Buck targets；
- 钉 Reindeer，并从 Cargo 事实源生成第三方 crate 规则；
- 覆盖当前 build、unit test 与 smoke test；
- 保留 Cargo 路径作为 parity oracle，直到 Buck 产物与行为验证一致。

### PR 3 · CI 切换

- PR 主检查改为 Buck2 build/test；
- Linux、macOS Intel 与 macOS Apple Silicon 使用明确的平台和执行环境；
- GitHub Actions 引用钉完整 commit SHA；
- 消除 feature branch 同时触发 push 与 pull request 的重复运行；
- Buck 检查稳定后成为 `main` 必需检查；Cargo parity 暂时保留但不再代表主构建入口；
- 外部子系统 native build 与整包生命周期测试保持独立 job，不能用绿色第一方 job 冒充完整发行验证。

### PR 4 · 发行组装与旧入口收口

- Buck2 导出第一方二进制集合与 manifest；
- 外部子系统继续导出运行包、源码包、checksum 与 manifest；
- 确定性发行组装器消费两侧产物，生成三个目标平台的用户安装包；
- macOS 签名与 notarization 放在确定性构建后的 CD 阶段；
- Release workflow 上传运行包、源码包、checksums、SBOM 与 provenance；
- 通过三平台 parity 和安装生命周期验证后，删除只为第一方旧编译入口存在的脚本；保留 Cargo manifests、外部子系统 native build 与全部运行期脚本。

## 明确不做

- 不把 Tuwunel 的 Cargo graph 生成成 HCTL2 BUCK targets；
- 不把 tmux、libevent、ncurses 或 utf8proc 改写成细粒度 Buck C/C++ targets；
- 不要求外部官方二进制与 HCTL2 使用同一编译器；
- 不为了 Buck2 重写已经通过生命周期验证的外部子系统脚本；
- 不把运行期启停脚本包装成编译规则；
- 不在 parity 成立之前一次性删除 Cargo 或现有发行路径。
- 不用 workflow 自制 fingerprint、Cargo 目录 cache 或最终依赖包 artifact 模拟 Buck action cache。
- 当前不采购托管 Remote Execution/Remote Action Cache；使用本机 loopback 标准 REAPI cache，macOS CI 只持久化其 CAS/action 数据目录。

## 工作量表达方式

AI 化开发不再用传统人日作为主要估算单位。后续采用三项粗粒度指标：

1. **改动量级**：本次属于中等基础设施变更，不改业务模型，但触及构建入口、三平台工具链、CI 与发行边界；
2. **风险面**：第一方 Rust 规则低，Buck2 开源工具链与 macOS 双架构中，签名/notarization 和严格 hermeticity 高；
3. **可独立验收批次**：四个主 PR，每批都有可运行命令和退出条件，验证通过后才进入下一批。

估算保持大略，不为尚未遇到的问题虚构精确步骤或日历时间。

## 总体验收

- 一条 Buck2 命令构建和测试全部 HCTL2 第一方代码；
- 三个平台的工具链和 target identity 可查询，未使用自造字符串隐式推断；
- Cargo/pnpm 等语言锁文件仍是依赖事实源，不出现两套手写图；
- 外部子系统继续用原生入口构建，Buck2 不侵入其内部；
- CI 分清第一方、外部子系统与完整发行三种健康状态；
- 从空 runner 产生三个可安装发行包，完成 checksum、安装、启动、状态、smoke 与停止验证；
- `remote/main` 只接受通过相应检查的 PR，工作树与发布产物边界保持干净。

## 实施记录

四批按顺序完成，后一批只从前一批合并后的 `remote/main` 开始：

- PR #15 固化本文；
- PR #16 建立 Buck2、DotSlash、Prelude、三平台定义和 Rust 1.98.0 工具链；
- PR #17 迁移 protocol、agentd、tool、单元测试、smoke 与 Clippy，并钉 Reindeer；
- PR #18 把 Linux、macOS Intel、macOS Apple Silicon 的主检查切到 Buck2，独立跑通三平台外部整包生命周期；
- 后续批次导出第一方二进制合同，加入确定性完整发行组装、SPDX、checksums、artifact attestation 与 tag 发布流水线；最终仍保持 `src` 为 workspace 与 `root` cell，发行资料以 `src/packaging/release/assets/` 下的受检快照进入声明输入。

第 3 批远端冷构建实测：Linux 外部整包 1 分 58 秒，Apple Silicon 26 分 35 秒，Intel Mac 37 分 9 秒；三者都完成离线安装、四执行面启动、状态、smoke 和停止。Intel 链接器可能产生旧式 `LC_VERSION_MIN_MACOSX`，校验器因此同时识别它与现代 `LC_BUILD_VERSION`；当时两种形式共同受 macOS 13 deployment target 上限约束。2026-08-28 改用最低系统为 15 的官方 `tmux-builds` Darwin 二进制后，产品基线升为 macOS 15，自主 tmux/libevent/ncurses/utf8proc 构建链一并删除。

最后一批在 Ubuntu 26.04 本机用既有 159,585,630 字节外部运行包组装出 162,026,924 字节完整包；连续两次组装的运行归档与 SPDX 均通过字节级 `cmp`。最终包随后通过第一方命令链接、幂等安装、Tuwunel、Cinny、Vikunja、Dagu、tmux 完整生命周期验证。三平台的同一验收由 Release workflow 在临时 GitHub-hosted runner 上复跑。

`main` 已要求 PR、最新主线、单一 `CI gate`、管理员不可绕过、对话已解决，并禁止 force-push 与删除；自合不要求额外审批。仓库还强制所有 GitHub Action 使用完整 commit SHA。`ubuntu-26.04` 是 GitHub-hosted public-preview 标准镜像，不是本机或长期虚拟机；公开仓库使用标准 Linux/Intel Mac/Apple Silicon runner 不计 Actions 分钟费用，缓存和 artifact 存储仍按仓库额度管理。

## 后续修正：外部构建进入 action graph

2026-08-28 的复盘确认，早期“Buck 只接收 workflow 预先生成的外部包”边界太弱：Buck 看不到发行包依赖 Tuwunel 构建，因此也无法用自己的 action key 判断是否需要重建。曾短暂提出用一份 shell fingerprint 和 GitHub artifact 复用最终依赖包，这会形成与 Buck 平行的身份系统，现已撤销。

修正后的边界如下：

- `packaging/dependencies/lock.json` 声明外部发行物、源码和 macOS Tuwunel Rust 组件的 URL 与 SHA-256；
- Buck `http_file`/`http_archive` 负责下载与摘要验证，只把当前 target 所需资产放进执行闭包；
- Linux 外部 action 解包官方二进制；macOS 外部 action 使用 Buck 声明的 Rust 1.95.0 工具链调用 Tuwunel 的 Cargo 构建，其余组件消费官方二进制；
- `packaging/dependencies:package` 把准备结果组装成运行包、源码包和 sidecar；`packaging/release:complete` 同时依赖该目录与第一方导出，因此一条 Buck 请求直接产出完整发行；
- `src/` 是唯一 workspace，许可证和中文使用说明的发行快照由 `root//packaging/release` 导出并成为组包 action 的声明输入；仓库级事实源与快照的一致性由 CI 静态合同检查。
- Code workflow 的 target pattern 明确限定在第一方应用、crate 与工具链探针；外部子系统只由 Release workflow 构建一次。依赖 workflow 在 PR 中只跑静态合同，main push 或手动触发时才独立复验三平台生命周期。

早期实现只在相同 `buckd` 下复用 `packaging/release:complete`，导致每个 worktree 和临时 GitHub runner 重编 macOS Tuwunel，已判定不可接受。现在由钉定的 `bazel-remote` 在本机 loopback 提供标准 REAPI CAS/action cache：各 worktree 保持独立 daemon 与 `buck-out`，只共享内容寻址结果；macOS CI 用 `actions/cache` 持久化同一 REAPI 数据目录，cache key 随 Buck、工具链和外部准备输入换代，并用前缀恢复上一代 CAS。Linux 只消费官方二进制，不承担这份缓存开销。这里没有恢复自制 fingerprint、Cargo 目录 cache 或最终依赖包 artifact 接力；将来若采购托管 REAPI，只需更换 endpoint 和凭证。

实现验收还发现，OSS Prelude 的 genrule 会把 `cacheable = True` 与本地执行 label 联合判断；只设置前者时准备 action 仍显示 `allows_cache_upload: false`。最终为会联网调用上游 Cargo 的独立 `tuwunel` action 标注 `network_access`，为预编译组件准备、工具链拼装和大目录组包 action 标注 `large_copy`，使它们保持本地执行并允许把确定性结果上传到标准 action cache。

## 后续收口：消除 shell-as-Make

2026-08-28 再次核对后发现的 shell-as-Make 已收口：第一方构建、外部制品下载、macOS Tuwunel 工具链、payload/源码组包、完整发行组装与生命周期测试都位于 Buck action graph。

落地后的边界：

- 外部运行包与源码包是 `packaging/dependencies:package` 的按平台目录输出；
- 完整运行包、源码包、SBOM、release manifest 和 checksums 是 `packaging/release:complete` 的目录输出；
- 包合同与离线生命周期由 `package-test` / `complete-test` 的 `sh_test` 执行；测试体保留 shell，输入、平台和被测产物由 Buck 声明；
- 只负责 `uname`、target 分发和重复版本配置的入口已删除；平台选择统一由 Buck configuration 与 `select` 完成，`lock.json` 是唯一外部构建 metadata 事实源；
- 继续保留用户安装后需要的 `install/start/stop/status/smoke`、平台 runtime hook，以及一次性 DotSlash/Reindeer 引导工具；这些不是构建编排重复实现；
- macOS Tuwunel 继续由粗粒度 action 调上游 Cargo，不把上游 crate 图复制成 HCTL2 rules；
- 不新增自制下载器、fingerprint、缓存协议或 workflow 产物接力。Buck 的下载规则、action key、声明输出和测试规则是唯一构建机制。

收口判据已经满足：一条按平台配置的 Buck 请求能产出最终发行文件；workflow 只负责选择平台、调用 Buck、上传/证明产物和发布，不再理解组包步骤。仓库中剩余脚本均归入“上游原生构建 action body”“随包运行期代码”“测试体”或“构建工具引导”，不再存在 shell 充当第二套构建图。

## 后续修正：恢复 `src/` 构建边界

最后一批曾为了直接读取仓库根 `LICENSE` 与 `docs/usage.md`，把 `.buckconfig`、`.buckroot` 和一个导出文件的 `BUCK` 提到仓库根。这是边界回归，不是 Buck2 原生机制的要求。2026-08-29 已撤销：Buck workspace 与所有 targets 回到 `src/`；发行资料使用 `src/packaging/release/assets/` 中由 Buck 声明的快照；CI 用 `cmp` 校验快照和仓库文档事实源。完整发行仍由单个 Buck 请求生成，没有恢复 Buck 外的组包阶段。

## 后续优化：隔离昂贵 action 与校验缓存合同

2026-08-29 进一步把原先单一 `prepared` 拆成六个组件 action。macOS `tuwunel` 只声明锁定源码、Rust 工具链、最小公共 action helper、Mach-O helper、独立原生构建体与由 `./buck2` 注入的 Xcode/SDK identity；输出只保留可执行文件、许可证、构建环境和非系统 dylib 闭包。其他五项各自只声明对应官方制品与必要配置，任一组件的制品、配置或专属脚本变化不再使其他组件失效。源码伴随归档由最终 `package` 直接引用 Buck 下载目标。不可变 action 内原先模拟增量目录的 marker 同时删除。

Apple Silicon 本机最终实测：`tuwunel` action 输出 77 MiB；Vikunja、Dagu、tmux、Cinny、Static Web Server 分别约 107、148、1.6、59、6 MiB，均不再保留同一大二进制的解压副本；依赖归档目录 176 MiB，完整发行目录 192 MiB。原单体准备输出约 855 MiB，其中包含完整 Cargo target/source tree 与全部下载；这些内容现已退出 action output。

CI 不再保存第一方 `buck-out` 或 DotSlash 下载目录。前者实测恢复 300–570 MiB 后仍有 0% Buck action hit，而三平台第一方冷构建只需约半分钟到两分钟。仅 macOS 外部构建保留标准 REAPI 数据目录：外层 key 包含 runner image 与昂贵 action 的受控输入；精确 key 恢复后必须从 Buck event log 机械确认 `tuwunel` 是 remote cache hit，否则 workflow 失败。Actionlint 与 ShellCheck 使用摘要锁定的官方单文件发行物，版本字段由 Buck test 校验 Cargo 与 Starlark 事实一致。
