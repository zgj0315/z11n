# 平台基础服务
## 1 agent
### 1.1 注册
服务端计算 agent_id，注册成功后，返回给 agent  
agent 将 agent_id 存储于本地文件和环境变量中  
如果这两个地方的 agent_id 相同，不需要注册
### 1.2 心跳
header 携带 agent_id，携带空消息与 Server 通信
Server 判断 agent_id 是否成功注册  
返回消息中，携带发送给 agent 的指令
### 1.3 主机信息
agent 采集主机信息，上报 Server
## 2 ui
### 2.1 Agent列表查询
tbl_agent 表中存储 agent 信息
### 2.2 Agent详情展示
sea-orm 操作 tbl_agent
### 2.3 Host列表查询
tbl_host 表中存储主机信息
### 2.4 Host详情展示
sea-orm 操作 tbl_host
### 2.5 主机信息更新
通知 agent 重新采集一遍主机信息
