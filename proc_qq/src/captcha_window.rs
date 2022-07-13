use std::collections::HashMap;

use wry::application::platform::windows::EventLoopExtWindows;
use wry::{
    application::{
        dpi::LogicalSize,
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::run_return::EventLoopExtRunReturn,
        window::WindowBuilder,
    },
    http::ResponseBuilder,
    webview::WebViewBuilder,
};

#[derive(Debug)]
enum UserEvents {
    CloseWindow(String),
}

pub(crate) fn ticket(url: &str) -> Option<String> {
    let mut ticket = None;
    let mut event_loop = EventLoop::<UserEvents>::new_any_thread();
    let proxy = event_loop.create_proxy();
    let ipcproxy = proxy.clone();
    let mut windows = HashMap::new();
    let window = WindowBuilder::new()
        .with_title("滑")
        .with_inner_size(LogicalSize {
            width: 455,
            height: 390,
        })
        .build(&event_loop)
        .unwrap();
    let windowid = window.id();
    let _webview = WebViewBuilder::new(window)
        .unwrap()
        .with_url(url)
        .unwrap()
        .with_devtools(true)
        .with_ipc_handler(move |_, s| {
            let _ = ipcproxy.send_event(UserEvents::CloseWindow(s));
        })
        .with_custom_protocol("ricq".into(), move |request| {
            let _ticket = String::from_utf8_lossy(request.body()).to_string();
            let _ = proxy.send_event(UserEvents::CloseWindow(_ticket));
            ResponseBuilder::new()
                .status(200)
                .body("ok".as_bytes().to_vec())
        })
        .with_initialization_script(
            r#"
            var origOpen = XMLHttpRequest.prototype.open;
            XMLHttpRequest.prototype.open = function () {
                this.addEventListener('load', function () {
                    if (this.responseURL == 'https://t.captcha.qq.com/cap_union_new_verify') {
                        var j = JSON.parse(this.responseText);
                        if (j.errorCode == '0') {
                            window.ipc.postMessage(j.ticket);
                            if (navigator.userAgent.indexOf('Windows') > -1) {
                                fetch('https://ricq.ticket', { 
                                    method: "POST",
                                    body: j.ticket
                                });
                            }
                        }
                    }
                });
                origOpen.apply(this, arguments);
            }
        "#,
        )
        .build()
        .unwrap();
    windows.insert(windowid, _webview);
    event_loop.run_return(|event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("启动滑块窗口"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::UserEvent(UserEvents::CloseWindow(x)) => {
                windows.remove(&windowid);
                ticket = Some(x);
                *control_flow = ControlFlow::Exit
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(x),
                ..
            } => {
                println!("{:?}", x);
            }
            _ => (),
        }
    });
    ticket
}
