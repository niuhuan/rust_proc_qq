RUST_PROC_QQ
============

[![license](https://img.shields.io/github/license/niuhuan/rust_proc_qq)](https://raw.githubusercontent.com/niuhuan/rust_proc_qq/master/LICENSE)
[![crates.io](https://img.shields.io/crates/v/proc_qq.svg)](https://crates.io/crates/proc_qq)

- Rust语言的QQ机器人框架. (基于[RICQ](https://github.com/lz1998/ricq))
- 开箱即用, 操作简单, 代码极简


QQ机器人框架 | [Telegram(电报)机器人框架](https://github.com/niuhuan/teleser-rs)

## 框架目的

- 简单化 : 让程序员写更少的代码
    - 自动管理客户端生命周期以及TCP重连
    - 封装登录流程, 自动获取ticket, 验证滑动条
- 模块化 : 让调理更清晰
    - 模块化, 实现插件之间的分离, 更好的启用禁用

# 设计思路

所有的功能都是由插件完成, 事件发生时, 调度器对插件轮训调用, 插件响应是否处理该事件, 直至有插件响应事件, 插件发生异常, 或插件轮训结束, 最后日志结果被记录, 事件响应周期结束。

![img.png](images/invoke.png)

## 如何使用 / demo

密码登录第一次需要滑块助手 https://github.com/mzdluo123/TxCaptchaHelper


新建一个rust项目, 并将rust环境设置为nightly

```shell
# 设置rust默认环境为 nightly
rustup default nightly

# 或

# 设置当前项目rust环境设置为 nightly
rustup override set nightly
```

### 引用

在Cargo.toml中引入proc_qq

```toml
proc_qq = "0.1"
```

### 声明一个模块

hello_module.rs

```rust
use proc_qq::re_exports::ricq::client::event::GroupMessageEvent;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent, MessageSendToSourceTrait,
    Module,
};

/// 监听群消息
/// 使用event宏进行声明监听消息
/// 参数为RICQ支持的任何一个类型的消息事件, 必须是引用.
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
        // .priority_session("session.token")      // 使用session.token登录
        .authentication(QRCode)                    // 若不成功则使用二维码登录
        // 注意，这里使用的设备必须支持二维码登录，例如安卓手表
        // 如果您使用为不支持的协议协议，则会登录失败，例如安卓QQ 
        // .authentication(UinPasswordMd5(config.account.uin, password)) // 账号密码登录
        .device(JsonFile("device.json")) // 设备默认值 
        .version(&ANDROID_WATCH)  // 安卓手表支持扫码登录
        // .show_slider_pop_menu_if_possible() // 密码登录时, 如果是windows, 弹出一个窗口代替手机滑块 (需要启用feature=pop_window_slider)
        .modules(vec![hello_module::module()])    // 您可以注册多个模块
        .show_rq(Some(ShowQR::OpenBySystem))  // 自动打开二维码 在macos/linux/windows中, 不支持安卓
        .build()
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
                .with_target("ricq", Level::DEBUG)
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

### 登录

```rust
    // .show_rq(Some(ShowQR::PrintToConsole))  // 打印二维码到控制台
    // .show_rq(ShowQR::Custom(Box::pin(|buff| {  // 自定义显示二维码
    //   Box::pin(async move {
    //     println!("buff : {:?}", buff.to_vec());
    //     Ok(())
    //   })
    // })))
```

### 支持的事件

```rust
use ricq::client::event::{
    DeleteFriendEvent, FriendMessageEvent, FriendMessageRecallEvent, FriendPokeEvent,
    NewFriendRequestEvent, GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent,
    GroupMuteEvent, GroupNameUpdateEvent, JoinGroupRequestEvent, KickedOfflineEvent, MSFOfflineEvent,
    NewFriendEvent, GroupTempMessageEvent,
};
use ricq::client::event::{
  GroupDisbandEvent,
  MemberPermissionChangeEvent,
  NewMemberEvent,
  SelfInvitedEvent,
};
use proc_qq::{MessageEvent, LoginEvent, ConnectedAndOnlineEvent, DisconnectedAndOfflineEvent, };
```

- MessageEvent: 同时适配多种消息
- LoginEvent: 登录成功事件 (RICQ中这个事件类型为i64,这里做了封装)
- ConnectedAndOnlineEvent: 连接成功, 并且登录后 (proc-qq状态)
- DisconnectedAndOfflineEvent: 掉线并且断开连接 (proc-qq状态)

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
MessageChainAppendTrait;

let chain: MessageChain;
let chain = chain.append(at).append(text).append(image);
```

## 事件结果

使用result_handlers监听处理结果 (事件参数正在开发)

```rust
use proc_qq::result;
use proc_qq::EventResult;

#[result]
async fn on_result(result: &EventResult) -> anyhow::Result<bool> {
    match result {
        EventResult::Process(info) => {
            tracing::info!("{} : {} : 处理了一条消息", info.module_id, info.handle_name);
        }
        EventResult::Exception(info, err) => {
            tracing::info!(
                "{} : {} : 遇到了错误 : {}",
                info.module_id,
                info.handle_name,
                err
            );
        }
    }
    Ok(false)
}
```

```rust
ClientBuilder::new()
.modules(vec![hello_module::module()])
.result_handlers(vec![result_handlers::on_result {}.into()])
.build()
```

## 定时任务或客户端事件发送消息

参考template, 使用run_client(Arc\<Client\>), 使得机器人与定时任务并行, 并使用rc_client发送消息

## 手动实现handler和原理

手动实现一个handler

```rust

/// 每个handler都是一个struct
struct OnMessage;

/// 给他实现一个Process, 它就对应着监听什么事件
#[async_trait]
impl MessageEventProcess for OnMessage {
    async fn handle(&self, event: &MessageEvent) -> anyhow::Result<bool> {
        self.do_some(event).await?;
        Ok(true)
    }
}

/// 实现一些其他的方法用于调用
impl OnMessage {
    async fn do_some(&self, _event: &MessageEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 将process转换成handler
fn on_message() -> ModuleEventHandler {
    ModuleEventHandler {
        name: "OnMessage".to_owned(),
        process: ModuleEventProcess::Message(Box::new(OnMessage {})),
    }
}

/// 将转化的方法名写到里面
pub(crate) fn module() -> Module {
    module!("hello", "你好", login, print, group_hello, on_message)
}
```

为什么要强调一下手动创造handler

```rust

async fn do_some(_event: &MessageEvent) -> anyhow::Result<()> {
    // 做一些线程不安全的事情
    Ok(())
}

#[event]
async fn handle(event: &MessageEvent) -> anyhow::Result<bool> {
    do_some(event).await?;  // 那么这里的引用生命周期有问题
    Ok(true)
}

```

总会遇到一些线程不安全的类, 例如*scraper*. 这个时候编译器会反复告诉你 "maybe used later". 您可以尝试使用手创造一个handler解决.

## 其他特性

    event参数
    MessageEvent / FriendMessageEvent / GroupMessageEvent / GroupTempMessageEvent
    trim_regexp trim_eq regexp eq all any not
    为什么会有trim: ricq获取消息会在最后追加空白字符
 
```rust
#[event(trim_regexp = "^a([\\S\\s]+)?$", trim_regexp = "^([\\S\\s]+)?b$")]
async fn handle2(event: &MessageEvent) -> anyhow::Result<bool> {
  event
          .send_message_to_source("a开头且b结束".parse_message_chain())
          .await?;
  Ok(true)
}

#[event(any(trim_regexp = "^a([\\S\\s]+)?$", trim_regexp = "^([\\S\\s]+)?b$"))]
async fn handle3(event: &MessageEvent) -> anyhow::Result<bool> {
  event
          .send_message_to_source("a开头或b结束".parse_message_chain())
          .await?;
  Ok(true)
}
```

## 其他

实现的功能请转到RICQ仓库查看, 本仓库仅为RICQ的框架.

RICQ 还在发展阶段, 迭代速度较快, 可能出现更改API的情况, 如遇无法运行, 请提issues.

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

##### 额外协议的说明

- 暂定本仓库开源协议与RICQ保持一致.
  - MPL 2.0
  - 如RICQ更换协议, 请以最新协议为准, 您可以提出ISSUE提醒我进行更新
- 仓库持有人在变更仓库协议时无需经过其他代码贡献者的同意, 您在PR时就代表您同意此观点

#### 鸣谢

- RICQ commiters 
- JetBrains IDEs

![](images/JetBrains.png)
