# 远程操控与会话同步单案

本目录保存「把本机 Harness 会话远程化/多端化」一族产品的逐对象审计(代码与 changelog 级,逐项钉定 commit/版本与许可)。跨对象归纳与 Codex Remote Feishu 主条目在上一级:[远程操控与会话同步](../remote-control.md)。这一族不拥有任务语义,均为 L1 邻近证据;复用决策与不采用边界见各文件。

单案(2026-08-30 审计):

- [MindFS](./mindfs.md):外部会话导入与「路径/大小/mtime」游标增量同步、双游标断点重放;工具审批全自动放行的反面证据;
- [Paseo](./paseo.md):⑤ 类最完整的公开协议/SDK 参照——时间线 epoch+seq 游标、跨厂商结构化审批、注意力在场路由、adapter 契约;
- [HAPI](./hapi.md):结构化/字节流双投影与「无观众不上传」门控、本地/远程互斥交接;
- [Happy](./happy.md):端到端加密多设备同步的活体参照(密文中继、配对即授钥、CAS 同步);
- [Remux](./remux.md):移动客户端「附着-定位-渲染-重连」的行为形状,tmux 原生 ID 为唯一路由身份;
- [Moshi](./moshi.md):注入前屏幕校验与在场感知提醒;与 Herdr 深度耦合的现成集成用法;
- [ServerCC](./servercc.md):厂商会话精确恢复与外部会话收养;「接管无控制权/交还语义」的空白写实;
- [QuickTUI](./quicktui.md):公开安装验证器 CI 门与能力自描述端点;qscreen 开源会话后端;
- [Redock](./redock.md):分阶段输入暂存区、BYO 语音引擎与 CJK;tmux/Herdr/psmux 可插拔持久层。

全量分类与复用结论见[研究总表](../README.md)。
