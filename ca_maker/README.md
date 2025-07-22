# Certification Authority For Cyber Kunlun
## 工程说明
- build-ca.sh 一键生成证书脚本
- home-ca/private 私钥目录
- home-ca/csrs 证书签名申请文件目录
- home-ca/crts 证书目录

## tonic框架证书说明
Tonic框架中，支持TLS证书，其中涉及到的三个文件制作方法  
文件地址：https://github.com/hyperium/tonic/tree/master/examples/data/tls  
工程地址：https://github.com/hyperium/tonic/tree/master/examples/src/tls

- ca.pem 签名证书，对应本工程中home-ca/crts/sub-ca.cert
- server.pem 服务端证书，对应本工程中home-ca/crts/zhaogj-ca.cert
- server.key 服务端私钥，对应本工程中home-ca/private/zhaogj-ca.key
