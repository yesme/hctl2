# HCTL1 / yesme/hctl

> 类别：⑦ 直接谱系 · 证据编号：E-L2-HCTL1<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l2-hctl1"></a>
## E-L2-HCTL1 · HCTL1 / yesme/hctl

HCTL1 是 HCTL2 L2 语义内核的直接前身，也是可执行的技术谱系证据；它不是外部复用来源，不能与 HCTL2 的原生语义核心混为一谈。审计固定在 [`main@3148042c`](https://github.com/yesme/hctl/tree/3148042cb2faf8df0dc8be92710b9468c8618516)（2026-07-28，Apache-2.0）。仓库没有标签或正式发布；README 表明 P1 内核已经进入主干，P2/P3 仍处于规划阶段。

它最独特的证据是一套不依赖守护进程和数据库的 Git 语义内核：每个 Seat 一条只追加事件引用、本地与远端 CAS、电平触发式对账、事实不完整时默认拒绝、Obligation/CLAIM 与 claim OID 隔离栅栏、精确匹配 `{base, head}` 的 Verdict、法定票数，以及携带事实摘要、无需依赖时钟即可重放的 squash merge Receipt。除规范外，仓库还提供可执行用例库，覆盖过期 Gate、权限、竞争、JCS 身份、组合法定票数、迟到 Finding、重新 Gate 时的结论沿用，以及初始化切换。

HCTL2 继承版本与证据、领取与隔离栅栏、法定票数、Receipt 和对账的思路，但不会原样继承其对象与事实源：

- HCTL1 的 `Seat = harness × model` 表示协作身份；HCTL2 的 Seat 是 Obligation 内的逻辑执行者或投票者位置，下挂 `0..N` 个 Attempt；
- HCTL1 的 Obligation 来自静态分派中的 author/gate/merge；HCTL2 的 Obligation 对应 Dagu 外部检查点的一次执行责任；
- HCTL1 把每个 Seat 的 ref、PR 和 squash Receipt 作为全局协调事实；HCTL2 把运行治理放入 SQLite 控制库，以 Git 保存共享且低频变化的定义和证据，并由 Dagu 保存机械工作流位置；
- HCTL1 的回收机制不等于候选方案降级，而且没有 Project Room、Task Board、Workflow Revision、Run、Attempt、进程/PTY 或外部系统同步；
- 单一人类信任、唯一合并协调者且容量为 1，以及把 PR 当作协作原子，只适用于它所定义的窄范围运行方式。

主要证据：

- [README 范围](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/README.md#L7-L27)；[METHOD 中的事实、Seat 与领取](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L27-L114)；[Gate、结论沿用与合并](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L108-L182)
- [派生引擎](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/derive/derive.go#L47-L124)；[CAS 与待处理状态恢复](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/store/store.go#L15-L191)；[Receipt 重放](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/receipt/receipt.go#L14-L187)
- [可执行用例库](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/tests/corpus/README.md#L1-L53)；[Apache-2.0 许可证](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/LICENSE)
