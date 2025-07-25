# z11n
## 1 概述
系统整体架构：Client + Server, Browser + Server
## 2 模块说明
### 2.1 client/z11n_agent
独立Agent，独立安装部署
### 2.2 client/z11n_sdk
被集成sdk，依附于三方应用
### 2.3 server/client_service
提供 Client 的通信接口
### 2.4 server/ui_service
提供 RESTful API 服务
提供 UI 服务
### 2.5 server/ui_web
前端源代码
## 3 功能说明
### 3.1 agent
#### 3.1.1 注册
服务端计算 agent_id，注册成功后，返回给 agent  
agent 将 agent_id 存储于本地文件和环境变量中  
如果这两个地方的 agent_id 相同，不需要注册
#### 3.1.2 心跳
header 携带 agent_id，携带空消息与 Server 通信
Server 判断 agent_id 是否成功注册  
返回消息中，携带发送给 agent 的指令
#### 3.1.3 主机信息
agent 采集主机信息，上报 Server
### 3.2 ui
#### 3.2.1 Agent列表查询
tbl_agent 表中存储 agent 信息
#### 3.2.2 Agent详情展示
sea-orm 操作 tbl_agent
#### 3.2.3 Host列表查询
tbl_host 表中存储主机信息
#### 3.2.4 Host详情展示
sea-orm 操作 tbl_host
#### 3.2.5 主机信息更新
通知 agent 重新采集一遍主机信息
## 4 开发备忘录
### TodoList
- [ ] 采集软件信息
- [ ] 调研如何将 Server 端发出的扫描数据包传递到 Agent 上
- [ ] 前端国际化方案调研
- [ ] 调研 handlebars 实现kv内容的显示
- [ ] 系统升级（升级包制作，上传升级包+升级动作）
### 20250725
- [x] ui_service 与 client_service 通信
- [x] 更新主机信息
### 20250724
- [x] 系统安装包制作脚本
- [x] 支持上传多种主机信息
- [x] disk and network
### 20250723
- [x] agent register
- [x] migration and entity init
- [x] agent online cache
- [x] token
- [x] restful api works
- [x] ui works
- [x] agent_id 稳定机制
- [x] agent 版本号读取
- [x] sysinfo 采集主机信息
- [x] agent 和 host 的存储结构
- [x] agent 列表查询
- [x] host 列表查询
- [x] 采集进程信息
- [x] 主机详情使用标签页
- [x] 主机详情展示
### 20250722
- [x] 启动项目，设计文档
- [x] 设计工程结构
- [x] 制作 TLS 证书
- [x] gRPC 服务端模块（proto文件费了些功夫）
- [x] gRPC 客户端模块（客户端认证证书波折了一些）
- [x] 通过 Interceptor 操作 Header 中的 agent_id, agent_version, token

## 5 调试说明
```
# 启动客户端接收模块
cd server/client_service
cargo watch -x run

# 启动客户端发送模块
cd client/z11n_agent
cargo run

# 启动 RESTful API 模块
cd server/ui_service
cargo watch -x run

# 启动前端模块
cd server/ui_web
npm run dev
```