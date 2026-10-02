# axum · AppService 的 HTTP 入口

> 状态：采用 · 复核：2026-09-28 · P2.2 己

## 定位与上游能力

[axum 0.8.9](https://docs.rs/axum/0.8.9/axum/) 提供 Router、请求体上限、状态注入与 Tokio listener 服务；与已有 tonic 共用 hyper / tower。本库锁文件已经间接包含该版本。Matrix 的请求结构、路径参数与响应仍由 [ruma](../sdk/matrix.md) 解析，不在路由器里重写协议。

## 候选与边界

直接用 hyper 要自己做路由与请求体限制；另启 Web 服务进程则多一个生命周期。axum 已在依赖树中，作为 control 内部的回环 HTTP 入口即可。只接 AppService，不取代 owner Unix socket，也不开放治理命令 HTTP API。AppService 的 hs_token 校验和持久接收在业务适配器里完成。

## 决定建议

采用并精确钉 **axum =0.8.9**，启用 http1、tokio、json；许可证 MIT。复用现有 Tokio runtime、原生路由和请求体上限，避免自写 HTTP 服务。运行时不依赖 Rust 开发工具。
