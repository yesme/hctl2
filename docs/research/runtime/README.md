# Agent / Terminal 运行服务单案

本目录保存单个 Agency/Terminal 运行服务的源码审计与专项验证。跨候选归纳仍在上一级：

- [Agent 运行服务候选复审](../agentd-runtime-candidates-20260829.md)：Termio、tty7、cmux、Pilotty、tmux 库和相邻产品的统一比较；
- [tmux 与候选对照](../tmux-runtime.md)：历史 tmux 方案以及 Zellij、shpool 对照；
- [远程操控与会话同步](../remote-control.md)：远程客户端与会话同步产品的横向清单。

单案：

- [Herdr](./herdr.md)：源码审计和设计借鉴；
- [Herdr 运行服务验证记录](./agency-runtime-validation-20260829.md)：当前选定实现的 API、行为、限制与 macOS footprint。

全量分类与复用结论见[研究总表](../README.md)。
