use proc_macro::TokenStream;

use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg};

/// debug = note expanded codes if env PROC_QQ_CODEGEN_DEBUG exists
macro_rules! emit {
    ($tokens:expr) => {{
        use devise::ext::SpanDiagnosticExt;
        let mut tokens = $tokens;
        if std::env::var_os("PROC_QQ_CODEGEN_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting proc_qq_codegen debug output")
                .note(tokens.to_string())
                .emit_as_item_tokens();

            tokens.extend(debug_tokens);
        }
        tokens.into()
    }};
}

/// event proc
#[proc_macro_error]
#[proc_macro_attribute]
pub fn event(_: TokenStream, input: TokenStream) -> TokenStream {
    // must append to async fn
    let method = parse_macro_input!(input as syn::ItemFn);
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // params check
    let params = &method.sig.inputs;
    if params.len() != 1 {
        abort!(&method.sig.span(), "必须有且只能有一个参数");
    };
    let param = params.first().unwrap();
    let param = match param {
        FnArg::Receiver(_) => abort!(&param.span(), "不支持self"),
        FnArg::Typed(pt) => pt,
    };
    let param_ty = param.ty.as_ref();
    let param_ty = quote! {#param_ty};
    let tokens = match param_ty.to_string().as_str() {
        "& GroupMessageEvent" => (
            quote! {::proc_qq::GroupMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessage},
            false,
        ),
        "& PrivateMessageEvent" => (
            quote! {::proc_qq::PrivateMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::PrivateMessage},
            false,
        ),
        "& TempMessageEvent" => (
            quote! {::proc_qq::TempMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::TempMessage},
            false,
        ),
        "& GroupRequestEvent" => (
            quote! {::proc_qq::GroupRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupRequest},
            false,
        ),
        "& FriendRequestEvent" => (
            quote! {::proc_qq::FriendRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendRequest},
            false,
        ),
        "& NewFriendEvent" => (
            quote! {::proc_qq::NewFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewFriendEvent},
            false,
        ),
        "& FriendPokeEvent" => (
            quote! {::proc_qq::FriendPokeEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendPoke},
            false,
        ),
        "& DeleteFriendEvent" => (
            quote! {::proc_qq::DeleteFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::DeleteFriend},
            false,
        ),
        "& GroupMuteEvent" => (
            quote! {::proc_qq::GroupMuteEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMute},
            false,
        ),
        "& GroupLeaveEvent" => (
            quote! {::proc_qq::GroupLeaveEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupLeave},
            false,
        ),
        "& GroupNameUpdateEvent" => (
            quote! {::proc_qq::GroupNameUpdateEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupNameUpdate},
            false,
        ),
        "& GroupMessageRecallEvent" => (
            quote! {::proc_qq::GroupMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessageRecall},
            false,
        ),
        "& FriendMessageRecallEvent" => (
            quote! {::proc_qq::FriendMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendMessageRecall},
            false,
        ),
        "& MessageEvent" => (
            quote! {::proc_qq::MessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::Message},
            true,
        ),
        t => abort!(
            param.span(),
            format!("未知的参数类型 {}, 请在文档中查看兼容的事件以及参数类型 https://github.com/niuhuan/rust_proc_qq", t),
        ),
    };
    let trait_name = tokens.0;
    let enum_name = tokens.1;
    let build_in = tokens.2;
    // gen token stream
    let ident = &method.sig.ident;
    let ident_str = format!("{}", ident);
    let build_struct = quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident {}
    };
    let build_trait = if build_in {
        let block = &method.block;
        quote! {
            #[::proc_qq::re_export::async_trait::async_trait]
            impl #trait_name for #ident {
                async fn handle(&self, event: #param_ty) -> ::proc_qq::re_export::anyhow::Result<bool> #block
            }
        }
    } else {
        quote! {
            #method
            #[::proc_qq::re_export::async_trait::async_trait]
            impl #trait_name for #ident {
                async fn handle(&self, event: #param_ty) -> ::proc_qq::re_export::anyhow::Result<bool> {
                    Ok(#ident(event).await?)
                }
            }
        }
    };
    let build_into = quote! {
        impl Into<::proc_qq::ModuleEventHandler> for #ident {
            fn into(self) -> ::proc_qq::ModuleEventHandler {
                ::proc_qq::ModuleEventHandler{
                    name: #ident_str.into(),
                    process: #enum_name(Box::new(self)),
                }
            }
        }
    };
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}
