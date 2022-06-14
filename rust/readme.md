kad 从数据库

1. 从数据库加载数据

如果没有数据，就读取种子

2. 如果有数据，就开始尝试连接

3. 一旦有连接成功的，就开始请求更多

4. 一旦找不到更多，就开始清理数据库，删除不在当前 kad 中的

---

从数据库中按区间读取，直到没有

kad 网络

定期 心跳 15 秒一次

心跳内容为 (小端编码 as u32) 响应内容为时间 (大端编码 as u32)

udp 老化时间 20 秒 // https://www.amobbs.com/thread-5649364-1-1.html

---

64 位 文件大小

如果解密失败，不响应

room
  id
  name
  public
  update

解码失败

公钥 - ip
ip - 公钥

a
a >> 1 << 1  -

lower = a
upper = a+1
n = 0

while n<128
  n+=1
  lower_next = a >> n << n
  range(lower_next,lower)
  lower = lower_next
  upper_next = a >> n << n + 2**n
  range(upper, upper_next)
  upper = upper_next

[使用 Rust 的 RocksDB](https://blog.petitviolet.net/post/2021-03-25/use-rocksdb-from-rust)

每个频道有一个频道秘钥
知道频道秘钥就可以在频道发言

节点
  kad 网络
用户
频道
  一样要构建 kad 网络
消息
  短文本
  转发
  评论
授信
  六个可用性最高，在线最长并且有可用授信的节点
  计量单位 KB - 小时

kad 网络数据库结构设计

每个频道有一个频道的公钥和私钥
发言用用户秘钥签名，再用频道私钥签名，用公钥记录订阅

kad 网络 最多有 30*8 = 240 个桶
用 rocksdb 存储

开头全部一样 ，结尾从 x0000 到 xx000，从 xx111 到 x1111

kad 网络

只需要查找 > 某 id 的节点

ip - 节点公钥
节点公钥 - ip
