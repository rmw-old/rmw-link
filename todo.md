1. 利用 kv 的 alive_addr 做定期检查和超时清理
2. 任务编号和回调，u32，初始为随机

每个请求有一个任务编号
任务编号是 u32.to_le_bytes()
任务编号第一个字母不为 0

// dbg!((i, (i + (1 << 24)).to_le_bytes()));
// task 超过 expire time 没响应就超时

#[enum_dispatch]
trait KnobControl {
    //...
}

#[enum_dispatch(KnobControl)]
enum Knob {
    LinearKnob,
    LogarithmicKnob,
}
