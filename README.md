# z11n
## 1 概述
系统整体架构：Client + Server, Browser + Server
## 2 模块说明
### 2.1 client/agent_service
独立Agent，独立安装部署
### 2.2 client/sdk_service
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
#### 3.1.3 列表查询
tbl_agent 表中存储 agent 信息
moka 中存储 agent 状态信息
#### 3.1.4 详情展示
sea-orm 操作 tbl_agent

## 3 技术调研
- [ ] 跨进程读取sqlite数据库问题
- [ ] 跨进程读取moka库问题
- [ ] 封装一个数据库(sql + kv)操作中间件，将数据库操作收口
