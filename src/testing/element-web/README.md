# Element Web 人工验收

HCTL2 的 Chatroom 解决方案由 Tuwunel 服务端和官方 Element Web 浏览器客户端共同组成。发行构建会锁定、校验并打包 Element Web 官方发行包；安装后由 `hctl2-services` 与其他依赖统一管理，不要求最终用户另外安装客户端、Python 或 Node.js。

Element Web 是用于 Matrix 互操作和人工查看的随包客户端。它不是 HCTL2 Workbench，不拥有 HCTL2 治理权限，也不是四个执行面之外的第五种依赖。

## 启动与验收

安装当前离线包后执行：

```bash
hctl2-services start
```

命令会打印 Chatroom 地址和本机 Tuwunel 注册 token。浏览器打开 `http://127.0.0.1:6168/`，依次验证：

1. 使用输出的 token 注册账号并登录；
2. 创建普通非加密房间；
3. 查看房间列表并收发消息；
4. 刷新页面或重启服务后重新连接；
5. 执行 `hctl2-services smoke`，确认客户端配置仍只指向随包 Tuwunel。

可以用 `hctl2-services stop element-web` 单独停止浏览器客户端；这不会停止 Tuwunel。再次执行 `hctl2-services start element-web` 会先确保 Tuwunel 已启动，再启动客户端。

## 测试边界

- 使用官方 [Element Web `v1.12.26`](https://github.com/element-hq/element-web/releases/tag/v1.12.26) 发行包，固定 SHA-256；发行文件、GPL 许可证和对应源码进入离线包，但不进入 Git。
- Tuwunel 测试配置禁止房间加密与 federation；请创建普通非加密房间。
- 这里验证注册、登录、建房、房间列表、消息收发和客户端重连，不验证 HCTL2 的 Room 治理、Context、Request、Receipt 或 Workbench 交互。
- Element Web 的 UI、账号数据和浏览器存储不成为 HCTL2 权威事实。
