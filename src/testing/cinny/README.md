# Cinny 人工验收

HCTL2 的 Chatroom 解决方案由 Tuwunel 服务端和 Cinny 浏览器客户端共同组成。发行构建会锁定、校验并打包 Cinny 官方 Web 发行包；安装后由 `hctl2-services` 与其他依赖统一管理，不要求最终用户另外安装客户端、Python 或 Node.js。

Cinny 用于 Matrix 互操作和人工查看。它不是 HCTL2 Workbench，不拥有 HCTL2 治理权限，也不是四个执行面之外的第五种依赖。

## 启动与验收

安装当前离线包后执行：

```bash
hctl2-services start
```

命令会打印 Chatroom 地址和本机 Tuwunel 注册 token。浏览器打开 `http://127.0.0.1:6168/`，确认 Homeserver 固定为 `http://127.0.0.1:6167`，然后依次验证：

1. 使用输出的 token 注册账号并登录；
2. 创建普通非加密房间；
3. 查看房间列表并收发消息；
4. 刷新页面或重启服务后重新连接；
5. 执行 `hctl2-services smoke`，确认客户端配置仍只指向随包 Tuwunel。

可以用 `hctl2-services stop cinny` 单独停止浏览器客户端；这不会停止 Tuwunel。再次执行 `hctl2-services start cinny` 会先确保 Tuwunel 已启动，再启动客户端。

## 测试边界

- 使用官方 [Cinny `v4.12.6`](https://github.com/cinnyapp/cinny/releases/tag/v4.12.6) Web 发行包，固定 SHA-256；发行文件、AGPL 许可证和对应源码进入离线发行物，但不进入 Git。
- 客户端配置关闭任意 homeserver，只允许连接随包 Tuwunel，并使用 hash router 适配内部静态文件服务。
- Tuwunel 测试配置禁止房间加密与 federation；请创建普通非加密房间。
- 这里验证注册、登录、建房、房间列表、消息收发和客户端重连，不验证 HCTL2 的 Room 治理、Context、Request、Receipt 或 Workbench 交互。
- Cinny 的 UI、账号数据和浏览器存储不成为 HCTL2 权威事实。
