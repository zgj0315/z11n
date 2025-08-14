## 4 开发备忘录
### TodoList
- [ ] 采集软件信息
- [ ] 调研如何将 Server 端发出的扫描数据包传递到 Agent 上
- [ ] 前端国际化方案调研
- [ ] 调研 handlebars 实现kv内容的显示
- [ ] 平台升级功能
- [ ] Agent升级功能
- [ ] 优化ui_web代码的警告
- [ ] 自定义logo和title
- [ ] 密码加密存储
### 20250814
- [x] 权限控制ui显示内容
- [x] 内置一个只读角色
- [x] 解决npm run build问题
- [x] 验证码登录
### 20250813
- [x] 角色返回数据结构优化，去掉is_owned
- [x] 角色更新
- [x] 整理auth，分门别类
- [x] 用户删除
- [x] 用户详情
- [x] 用户创建
- [x] 用户更新
### 20250811
- [x] 用户，角色权限控制
### 20250807
- [x] 合并ui_service和client_service
### 20250804
- [x] bin文件输出版本号
- [x] server deploy to 172.16.104.97
### 20250802
- [x] llm task broker ui
- [x] llm task broker agent debug
### 20250801
- [x] llm task broker agent
### 20250728
- [x] llm task broker gRPC api code review
### 20250727
- [x] llm task broker design
- [x] llm task broker gRPC api
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
