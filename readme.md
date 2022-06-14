## 连接流程

1.
  1 ping
  30 公钥
  8  本地时间

  1+30+8 = 39

2.
  响应
  1 ping
  16 hash(私钥的哈希+本地时间+远程 ip 地址+远程公钥)
  8 本地时间

  1+16+8 = 25

3.
  1 ping
  30 公钥
  64 对远程哈希的签名
  24 远程哈希 + 远程时间
  * token
  1+24+30+64+* >= 119
  确保 xxhash64(远程哈希 + 远程时间 + token) leading_zero >= 8

4.
  1 ping
  30 自己的公钥
  16 公共秘钥的哈希
  1+30+16 = 47

## 数据结构

node  物理节点
channel 频道

每个频道有一个公钥和私钥
每个频道可以签名谁可以以这个身份发言
身份有：所有者 owner、管理员 admin、编辑 editor、订阅者 user
每个人都有一个默认频道，默认频道的公钥和私钥等于节点的公钥和私钥，默认频道就是频道的所有者
所有者可以签名授权其他用户身份，可以有多个所有者
这样设计的好处是，一个人可以有多个设备
群其实就是一个订阅合集，这样大家可以自由的屏蔽群中的某个人

会构建每个 channel 的 kad 网络。
默认会加入 rmw 的 kad 网络，作为 dns kad。

## 传输流程

心跳包 (谁发起 ping 谁发心跳)
  heartbeat 本地时间
  heartbeat 时间

数据大于 8 字节
解密失败响应 DecryptionFail
