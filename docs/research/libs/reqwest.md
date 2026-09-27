# reqwest · Matrix 出站请求

> 状态：采用 · 复核：2026-09-28 · P2.2 己

## 定位与上游能力

[reqwest 0.13.4](https://docs.rs/reqwest/0.13.4/reqwest/) 提供 HTTP 客户端、连接复用、超时与重定向策略。与 [Matrix 调研](../sdk/matrix.md) 的选型一致：ruma 构造和解析类型化请求，reqwest 只运送 HTTP 字节。

## 候选与边界

直接拼 curl 命令要另管进程与凭据；直接使用 hyper 要自行配置连接池。己包接随包的回环 Tuwunel，不接远程 homeserver，因此关闭默认 feature，仅启用 blocking，拒绝非回环地址；不额外拉 TLS/C 编译链。远程接入需要另开 TLS 能力，不能把这个本地端口误报为远程可用。阻塞客户端在已有 spawn_blocking 工作线程中使用，不阻塞 Tokio runtime，也不在网络调用期间占 Store 锁。禁用代理与重定向，防止 AppService 令牌被发往别处。

## 决定建议

采用并精确钉 **reqwest =0.13.4**，default-features=false、blocking；许可证 MIT OR Apache-2.0。设置有限超时；写失败按原意图回读，不由通用 HTTP 层盲重试。429 保留可重试错误，Matrix 事务键由上层固定。
