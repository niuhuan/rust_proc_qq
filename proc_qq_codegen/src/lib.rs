use proc_macro::TokenStream;

use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, FnArg, Meta, NestedMeta, Token};

use crate::event_arg::*;

mod event_arg;

/// debug = note expanded codes if env PROC_QQ_CODEGEN_DEBUG exists

macro_rules! emit {
    ($tokens:expr) => {{
        use proc_macro2_diagnostics::SpanDiagnosticExt;
        let mut tokens = $tokens;
        if std::env::var_os("PROC_QQ_CODEGEN_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting PROC_QQ code generation debug output")
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
pub fn event(args: TokenStream, input: TokenStream) -> TokenStream {
    // event attrs
    let attrs = parse_macro_input!(args as syn::AttributeArgs);
    let all: Vec<EventArg> = parse_args(attrs);
    // method
    let method = parse_macro_input!(input as syn::ItemFn);
    // process = bot_command
    let mut bot_command = None;
    let mut _all = vec![];
    for x in all {
        if let EventArg::BotCommand(command) = x {
            if bot_command.is_none() {
                bot_command = Some(command);
            } else {
                abort!(
                    &method.sig.span(),
                    "bot_command 只能有一个，切必须是直接写在event括号种"
                );
            }
        } else {
            _all.push(x);
        }
    }
    let all = _all;
    if contains_bot_command(&all) {
        abort!(
            &method.sig.span(),
            "bot_command 只能有一个，切必须是直接写在event括号种"
        );
    }
    let bot_command_info = if let Some(bot_command) = &bot_command {
        let bot_command_regexp =
            regex::Regex::new("^(\\S+)((\\s+\\{\\S+\\})+)?$").expect("proc_qq的正则不正确(1)");
        if !bot_command_regexp.is_match(bot_command) {
            abort!(
                &method.sig.span(),
                "bot_command 不符合规则： 您需要写成\"命令 {参数} {参数} ...\"的格式，例如：bot_command=\"/ban {user} {time}\"，命令不一定要使用/开始，这里只是演示。正则 \"^(\\S+)((\\s+\\{\\S+\\})+)?$\""
            );
        }
        let blank_sp_regexp = regex::Regex::new("\\s+").expect("proc_qq的正则不正确(2)");
        let sp: Vec<&str> = blank_sp_regexp.split(bot_command).collect();
        let command_name = sp.first().unwrap().to_string();
        let bot_command_regexp =
            regex::Regex::new("^[A-Za-z_]([A-Za-z0-9_]+)?$").expect("proc_qq的正则不正确(2)");
        let skip = sp.iter().skip(1).map(|i| i.to_string());
        let mut args = vec![];
        for x in skip {
            let x = x[1..(x.len() - 1)].to_string();
            if !bot_command_regexp.is_match(&x) {
                abort!(
                    &method.sig.span(),
                    "bot_command 的参数名只能由[0-9A-Za-z_]组成且不能以数字开头 : {}",
                    x
                );
            }
            if args.contains(&x) {
                abort!(&method.sig.span(), "bot_command 的参数名不能重复");
            }
            args.push(x);
        }
        Some((command_name, args))
    } else {
        None
    };
    // must append to async fn
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // params check
    let sig_params = &method.sig.inputs;
    if sig_params.is_empty() {
        abort!(&sig_params.span(), "需要事件作为参数");
    }
    let params: Vec<&FnArg> = sig_params.iter().collect();
    let (event_param, param_skip) = {
        let first_param = params.first().unwrap();
        if let FnArg::Receiver(_) = first_param {
            if params.len() == 1 {
                abort!(&first_param.span(), "需要事件作为参数");
            }
            (*params.get(1).unwrap(), 2)
        } else {
            (*first_param, 1)
        }
    };
    // 对事件进行匹配
    let event_param = match event_param {
        FnArg::Receiver(_) => abort!(&event_param.span(), "不支持self"),
        FnArg::Typed(pt) => pt,
    };
    let param_pat = event_param.pat.as_ref();
    let param_ty = event_param.ty.as_ref();
    let param_ty = quote! {#param_ty};
    let tokens = match param_ty.to_string().as_str() {
        "& LoginEvent" => (
            quote! {::proc_qq::LoginEventProcess},
            quote! {::proc_qq::ModuleEventProcess::LoginEvent},
        ),
        "& GroupMessageEvent" => (
            quote! {::proc_qq::GroupMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessage},
        ),
        "& FriendMessageEvent" => (
            quote! {::proc_qq::FriendMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendMessage},
        ),
        "& GroupTempMessageEvent" => (
            quote! {::proc_qq::GroupTempMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupTempMessage},
        ),
        "& JoinGroupRequestEvent" => (
            quote! {::proc_qq::JoinGroupRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::JoinGroupRequest},
        ),
        "& NewFriendRequestEvent" => (
            quote! {::proc_qq::NewFriendRequestEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewFriendRequest},
        ),
        "& NewFriendEvent" => (
            quote! {::proc_qq::NewFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewFriendEvent},
        ),
        "& FriendPokeEvent" => (
            quote! {::proc_qq::FriendPokeEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendPoke},
        ),
        "& DeleteFriendEvent" => (
            quote! {::proc_qq::DeleteFriendEventProcess},
            quote! {::proc_qq::ModuleEventProcess::DeleteFriend},
        ),
        "& GroupMuteEvent" => (
            quote! {::proc_qq::GroupMuteEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMute},
        ),
        "& GroupLeaveEvent" => (
            quote! {::proc_qq::GroupLeaveEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupLeave},
        ),
        "& GroupNameUpdateEvent" => (
            quote! {::proc_qq::GroupNameUpdateEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupNameUpdate},
        ),
        "& GroupMessageRecallEvent" => (
            quote! {::proc_qq::GroupMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupMessageRecall},
        ),
        "& FriendMessageRecallEvent" => (
            quote! {::proc_qq::FriendMessageRecallEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendMessageRecall},
        ),
        "& MessageEvent" => (
            quote! {::proc_qq::MessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::Message},
        ),
        "& MSFOfflineEvent" => (
            quote! {::proc_qq::MSFOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::MSFOffline},
        ),
        "& KickedOfflineEvent" => (
            quote! {::proc_qq::KickedOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::KickedOffline},
        ),
        "& ConnectedAndOnlineEvent" => (
            quote! {::proc_qq::ConnectedAndOnlineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::ConnectedAndOnline},
        ),
        "& DisconnectedAndOfflineEvent" => (
            quote! {::proc_qq::DisconnectedAndOfflineEventProcess},
            quote! {::proc_qq::ModuleEventProcess::DisconnectedAndOffline},
        ),
        "& GroupDisbandEvent" => (
            quote! {::proc_qq::GroupDisbandEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupDisband},
        ),
        "& MemberPermissionChangeEvent" => (
            quote! {::proc_qq::MemberPermissionChangeEventProcess},
            quote! {::proc_qq::ModuleEventProcess::MemberPermissionChange},
        ),
        "& NewMemberEventEvent" => (
            quote! {::proc_qq::NewMemberEventEventProcess},
            quote! {::proc_qq::ModuleEventProcess::NewMemberEvent},
        ),
        "& SelfInvitedEventEvent" => (
            quote! {::proc_qq::SelfInvitedEventEventProcess},
            quote! {::proc_qq::ModuleEventProcess::SelfInvitedEvent},
        ),
        "& GroupAudioMessageEvent" => (
            quote! {::proc_qq::GroupAudioMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupAudioMessageEvent},
        ),
        "& FriendAudioMessageEvent" => (
            quote! {::proc_qq::FriendAudioMessageEventProcess},
            quote! {::proc_qq::ModuleEventProcess::FriendAudioMessageEvent},
        ),
        "& ClientDisconnect" => (
            quote! {::proc_qq::ClientDisconnectProcess},
            quote! {::proc_qq::ModuleEventProcess::ClientDisconnect},
        ),
        t => abort!(
            event_param.span(),
            format!("未知的参数类型 {}, 事件必须作为&self下一个参数(或第一个参数), 请在文档中查看兼容的事件以及参数类型 https://github.com/niuhuan/rust_proc_qq", t),
        ),
    };
    let mut command_pats: Vec<(&syn::Pat, &syn::Type)> = vec![];
    let command_params = &params[param_skip..params.len()];
    if command_params.is_empty() && bot_command.is_none() {
    } else if command_params.len() > 0 && bot_command.is_none() {
        abort!(sig_params.span(), "您没有使用bot_command, 不支持更多的参数",);
    } else if bot_command.is_some()
        && command_params.len() != bot_command_info.as_ref().unwrap().1.len()
    {
        abort!(
            sig_params.span(),
            "当您使用了bot_command时，若bot_command有变量，函数需要追加的参数，且个数必须与bot_command的变量相等，且名称的顺序一致。例如 \"/ban {user} {time}\"，您需要追加两个参数，user和time，且必须user在前，顺序不能打乱",
        );
    } else {
        for i in 0..command_params.len() {
            let bot_arg_name = bot_command_info.as_ref().unwrap().1[i].as_str();
            let command_param = command_params[i];
            let command_param = match command_param {
                FnArg::Receiver(_) => abort!(&command_param.span(), "不支持self"),
                FnArg::Typed(pt) => pt,
            };
            let pat = command_param.pat.as_ref();
            let command_param_arg_name = format!("{}", quote! {#pat});
            if !command_param_arg_name.eq(bot_arg_name) {
                abort!(
                    sig_params.span(),
                    "函数追加的参数必须与bot_command的变量相等，且名称顺序一致 : \"{}\" != \"{}\"",
                    bot_arg_name,
                    command_param_arg_name,
                );
            }
            let ty = command_param.ty.as_ref();
            let command_param_arg_type = format!("{}", quote! {#ty});
            match command_param_arg_type.as_str() {
                "& str" => {
                    command_pats.push((pat, ty));
                }
                "String" => {
                    command_pats.push((pat, ty));
                }
                "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128"
                | "isize" | "usize" => {
                    command_pats.push((pat, ty));
                }
                _ => {
                    abort!(
                        &command_param.span(),
                        "不支持的类型 : {}",
                        command_param_arg_type
                    );
                }
            }
        }
    };
    //
    let trait_name = tokens.0;
    let enum_name = tokens.1;
    // gen token stream
    let ident = &method.sig.ident;
    let ident_str = format!("{}", ident);
    let build_struct = quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident {}
    };
    // gen trait
    let block = &method.block;
    let build_trait = if all.is_empty() && bot_command.is_none() {
        quote! {
            #[::proc_qq::re_exports::async_trait::async_trait]
            impl #trait_name for #ident {
                async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
            }
        }
    } else {
        match param_ty.to_string().as_str() {
            "& MessageEvent" => (),
            "& GroupMessageEvent" => (),
            "& FriendMessageEvent" => (),
            "& GroupTempMessageEvent" => (),
            _ => abort!(
                &method.sig.span(),
                "event 的参数只支持消息类型事件 (MessageEvent,*MessageEvent)"
            ),
        }
        let args_vec = args_to_token(all);
        if bot_command.is_none() {
            quote! {
                #[::proc_qq::re_exports::async_trait::async_trait]
                impl #trait_name for #ident {
                    async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> {
                        if !::proc_qq::match_event_args_all(#args_vec, #param_pat.into())? {
                            return Ok(false);
                        }
                        self.raw(#param_pat).await
                    }
                }
                impl #ident {
                    async fn raw(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
                }
            }
        } else {
            let mut p_pats = quote! {};
            let mut command_params_in_raw = quote! {};
            let mut gets = quote! {};
            let args_number = command_pats.len();
            let mut idx: usize = 0;
            for x in command_pats {
                let pat = x.0;
                let ty = x.1;
                p_pats.append_all(quote! {
                   #pat,
                });
                command_params_in_raw.append_all(quote! {
                   #pat: #ty,
                });
                gets.append_all(quote! {
                    let #pat: #ty = match ::proc_qq::CommandMatcher::new(params.get(#idx).unwrap()).get() {
                        Ok(value) => value,
                        Err(_) => return Ok(false),
                    };
                });
                idx += 1;
            }
            if !gets.is_empty() {
                gets = quote! {
                     use ::proc_qq::BlockSupplier;
                     #gets
                }
            }
            let command_name = bot_command_info.as_ref().unwrap().0.as_str();
            quote! {
                #[::proc_qq::re_exports::async_trait::async_trait]
                impl #trait_name for #ident {
                    async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> {
                        if !::proc_qq::match_event_args_all(#args_vec, #param_pat.into())? {
                            return Ok(false);
                        }
                        // 匹配指令是否能对应
                        let hand: ::proc_qq::HandEvent = #param_pat.into();
                        let content = hand.content()?;
                        let (matched, params) = ::proc_qq::match_command(
                            &content,
                            #command_name,
                        )?;
                        if !matched || #args_number != params.len() {
                            return Ok(false);
                        }
                        #gets
                        self.raw(#param_pat, #p_pats).await
                    }
                }
                impl #ident {
                    async fn raw(&self, #param_pat: #param_ty, #command_params_in_raw) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
                }
            }
        }
    };
    // gen into
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
    // emit
    emit!(quote! {
        #build_struct
        #build_trait
        #build_into
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn result(_: TokenStream, input: TokenStream) -> TokenStream {
    // must append to async fn
    let method = parse_macro_input!(input as syn::ItemFn);
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // params check
    let params = &method.sig.inputs;
    if params.len() != 1 && params.len() != 2 {
        abort!(&method.sig.span(), "必须有且只能有1~2个参数");
    };
    if params.len() == 1 {
        let pm = params.first().unwrap();
        let pm = match pm {
            FnArg::Receiver(_) => abort!(&pm.span(), "不支持self"),
            FnArg::Typed(pt) => pt,
        };
        let pm_ty = pm.ty.as_ref();
        let pm_ty = quote! {#pm_ty};
        if !pm_ty.to_string().as_str().eq("& EventResult") {
            abort!(&pm.span(), "一个参数时只支持 &EventResult");
        }
        let pm_pat = pm.pat.as_ref();
        // gen token stream
        let ident = &method.sig.ident;
        let ident_str = format!("{}", ident);
        let block = &method.block;
        return emit!(quote! {
            #[allow(non_camel_case_types)]
            pub struct #ident {}
            #[::proc_qq::re_exports::async_trait::async_trait]
            impl ::proc_qq::OnlyResultHandler for #ident {
                async fn handle(&self, #pm_pat: #pm_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
            }
            impl Into<::proc_qq::EventResultHandler> for #ident {
                fn into(self) -> ::proc_qq::EventResultHandler {
                    ::proc_qq::EventResultHandler{
                        name: #ident_str.into(),
                        process: ::proc_qq::ResultProcess::OnlyResult(Box::new(self)),
                    }
                }
            }
        });
    }
    abort!(method.span(), "现在只支持一个参数");
}

struct ModuleParams {
    span: Span,
    expressions: Vec<String>,
}

impl Parse for ModuleParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let params = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
        Ok(ModuleParams {
            span,
            expressions: params
                .into_iter()
                .map(|param| param.to_token_stream().to_string())
                .collect(),
        })
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn module(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as ModuleParams);
    if params.expressions.len() < 2 {
        abort!(params.span, "参数数量不足")
    }
    let id = syn::parse_str::<Expr>(&params.expressions[0]).expect("id 解析错误");
    let name = syn::parse_str::<Expr>(&params.expressions[1]).expect("name 解析错误");
    let mut handle_builder = String::new();
    for i in 2..params.expressions.len() {
        handle_builder.push_str(&format!("{} {{}}.into(),", params.expressions[i]));
    }
    let handle_invoker =
        syn::parse_str::<Expr>(&format!("vec![{handle_builder}]")).expect("handle invoker解析错误");
    TokenStream::from(quote! {
        ::proc_qq::Module {
            id: #id.to_owned(),
            name: #name.to_owned(),
            handles: #handle_invoker,
        }
    })
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn event_fn(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as syn::AttributeArgs);
    let method = parse_macro_input!(input as syn::ItemFn);
    let mut struct_names = vec![];
    for attr in attrs {
        let struct_name = match attr {
            NestedMeta::Meta(me) => match me {
                Meta::Path(p) => {
                    if p.segments.len() != 1 {
                        abort!(&p.span(), "格式为event_fn(ident1,ident2,...)");
                    }
                    p.segments.first().unwrap().ident.clone()
                }
                _ => abort!(&me.span(), "格式为event_fn(ident1,ident2,...)"),
            },
            NestedMeta::Lit(_) => abort!(&attr.span(), "格式为event_fn(ident1,ident2,...)"),
        };
        struct_names.push(struct_name);
    }
    let params = &method.sig.inputs;
    let mut params_token_stream = quote! {&self};
    for param in params {
        match param {
            FnArg::Receiver(_) => (),
            FnArg::Typed(pt) => {
                let param_pat = pt.pat.as_ref();
                let param_ty = pt.ty.as_ref();
                params_token_stream.append_all(quote! {, #param_pat: #param_ty })
            }
        }
    }
    let vis = method.vis;
    let asyncness = method.sig.asyncness;
    let method_name = method.sig.ident;
    let block = &method.block;
    let mut result = quote! {};
    for struct_name in struct_names {
        result.append_all(quote! {
            impl #struct_name {
                #vis #asyncness fn #method_name ( #params_token_stream ) #block
            }
        });
    }
    emit!(result)
}
