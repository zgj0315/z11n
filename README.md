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
### 3.1 [平台基础服务](./docs/platform_service.md)
### 3.2 [大语言模型推理服务中介](./docs/llm_task_broker.md)
## 4 [开发备忘录](./docs/dev_memo.md)
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

# 初始化前端模块
npm create vite@latest ui_web -- --template react-ts

# 启动前端模块
cd server/ui_web
npm run dev
```