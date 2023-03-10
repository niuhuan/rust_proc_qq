网络代理
=======

### 1. 修改`Cargo.toml`, 启用proc_qq的proxy-feature

```toml
proc_qq = { version = "0.1", features = ["proxy"] }
```

### 2. 使用 `connect_handler` + `proxy_by_url` 进行代理设置

代理设置为空字符串时不使用代理，否则会使用代理进行连接，目前只支持socks5

```rust
use proc_qq::ClientBuilder;
use proc_qq::proxy_by_url;

fn main() {
    ClientBuilder::new()
        .connect_handler(proxy_by_url("socks5://localhost:1080/".to_string()).unwrap())
}
```
