RUST_PROC_QQ
============

- Rust语言的QQ机器人框架. (基于[RS-QQ](https://github.com/lz1998/rs-qq))
- 开箱即用, 操作简单, 代码极简

## 框架目的

- 简单化 : 让程序员写更少的代码
    - 自动管理客户端生命周期以及TCP重连
    - 封装登录流程, 自动获取ticket, 验证滑动条
- 模块化 : 让调理更清晰
    - 模块化, 实现插件之间的分离, 更好的启用禁用

## 如何使用 / demo

新建一个rust项目, 并将rust环境设置为nightly

```shell
# 设置rust默认环境为 nightly
rustup default nightly
# 设置当前项目rust环境设置为 nightly
rustup override set nightly
```

### 引用

在Cargo.toml中引入proc_qq

```toml
proc_qq = { git = "https://github.com/niuhuan/rust_proc_qq.git", branch = "master" }
```

### 声明一个模块

hello_module.rs

```rust
use proc_qq::re_exports::rs_qq::client::event::GroupMessageEvent;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent, MessageSendToSourceTrait,
    Module,
};

/// 监听群消息
/// 使用event宏进行声明监听消息
/// 参数为rs-qq支持的任何一个类型的消息事件, 必须是引用.
/// 返回值为 anyhow::Result<bool>, Ok(true)为拦截事件, 不再向下一个监听器传递
#[event]
async fn print(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.eq("你好") {
        event
            .send_message_to_source("世界".parse_message_chain())
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        event
            .send_message_to_source("NB".parse_message_chain())
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[event]
async fn group_hello(_: &GroupMessageEvent) -> anyhow::Result<bool> {
    Ok(false)
}

/// 返回一个模块 (向过程宏改进中)
pub(crate) fn module() -> Module {
    // id, name, [plugins ...]
    module!("hello", "你好", print, group_hello)
}
```

### 启动

main.rs

```rust
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::Authentication::{QRCode, UinPassword};
use proc_qq::ClientBuilder;

mod hello_module;

/// 启动并使用为二维码登录
#[tokio::test]
async fn test_qr_login() {
    // 初始化日志打印
    init_tracing_subscriber();
    // 使用builder创建
    ClientBuilder::new()
        .priority_session("session.token")      // 默认使用session.token登录
        // .device(JsonFile("device.json")) // 设备默认值 
        .authentication(QRCode)                 // 若不成功则使用二维码登录
        .build(vec![hello_module::module()])    // 您可以注册多个模块
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
}

fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("rs_qq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                // 这里改成自己的crate名称
                .with_target("proc_qq_examples", Level::DEBUG),
        )
        .init();
}

```

### 效果

![demo](images/demo_01.jpg)

## 功能

### 支持的事件

```rust
use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, FriendMessageEvent, TempMessageEvent,
};
use proc_qq::{MessageEvent, LoginEvent, };
```

支持更多种事件封装中...

### 拓展

#### 直接获取消息的正文内容

```rust
use prco_qq::MessageContentTrait;
MessageEvent::message_content;
```

#### 直接回复消息到消息源

```rust
Client::send_message_to_source;
Event::send_message_to_source;
Event::send_audio_to_source;
```

#### 直接将单个消息文字/图片当作MessageChain使用

```rust
MessageChainParseTrait;

client
.send_group_message(group_code, "".parse_message_chain())
.await?;
```

#### 

MessageChain链式追加

```rust
MessageChainTrait;

let chain: MessageChain;
let chain = chain.append(at).append(text).append(image);
```

## 其他

实现的功能请转到RS-QQ仓库查看, 本仓库仅为RS-QQ的框架.

RS-QQ 还在发展阶段, 迭代速度较快, 可能出现更改API的情况, 如遇无法运行, 请提issues.

#### [Examples](proc_qq_examples) 中提供了HelloWorld

#### [Template](proc_qq_template) 是一个机器人模版, 并提供了一些模块

##### 模版中封装了一些常用功能

直接回复文字, 如果是在群中会自动@
```rust
event.reply_text("你好").await?;
```
![](images/img_lib_01.png)

![](images/img_lib_02.png)

![](images/group_admin_01.png)

##### 数据库的说明

模版中使用了redis作为缓存, mongo作为数据库. 两个数据源搭建都非常简单.
- redis: 先下载[源码](https://redis.io/download), make, 运行 ./redis-server
- mongo: 下载[安装包](https://www.mongodb.com/try/download/community), 运行 ./mongod

如不需要, 请将database删除, 删除引用它的module, 最后删除main.rs中的init_mongo和init_redis.

##### 额外依赖的说明

模版中演示了如何发送语音消息

每日英语模块需要运行环境已经安装ffmpeg命令, 并且依赖silk-rs, 编译silk-rs需要libclang.dll.

- 下载LLVM-${version}-win64.exe并安装 : https://github.com/llvm/llvm-project/releases/ 
- 下载ffmpeg : https://www.ffmpeg.org/download.html
