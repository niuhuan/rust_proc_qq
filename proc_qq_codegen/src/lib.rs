use proc_macro::TokenStream;

use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, FnArg, Meta, NestedMeta, PatType, Token};

#[cfg(feature = "event_args")]
use crate::bot_command::*;
#[cfg(feature = "event_args")]
use crate::event_arg::*;

#[cfg(feature = "event_args")]
mod bot_command;
#[cfg(feature = "event_args")]
mod event_arg;

/// 如果设置PROC_QQ_CODEGEN_DEBUG变量，编译时将会以note方式打印PROC_QQ_CODEGEN的生成结果

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
    // 获取方法
    let method = parse_macro_input!(input as syn::ItemFn);
    #[cfg(not(feature = "event_args"))]
    if !args.is_empty() {
        abort!(&method.span(), "event参数请配合event_args特性使用");
    }
    // 判断是否为async方法
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // 判断事件
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
    let (trait_name, enum_name) = struct_name(event_param, param_ty.to_string());
    // event过程宏的的参数机型匹配
    #[cfg(feature = "event_args")]
    let attrs = parse_macro_input!(args as syn::AttributeArgs);
    #[cfg(feature = "event_args")]
    let (all_filter_without_bot_command, bot_command) = parse_args_and_command(&method, attrs);
    #[cfg(feature = "event_args")]
    let command_items = parse_bot_command(&method, bot_command);
    #[cfg(feature = "event_args")]
    let bot_args = parse_bot_args(&method, &params[param_skip..params.len()], command_items);
    #[cfg(not(feature = "event_args"))]
    if params.len() > param_skip {
        abort!(
            &params[param_skip].span(),
            "不支持更多的参数，请配合event_args特性使用"
        );
    }
    // struct
    let ident = &method.sig.ident;
    let ident_str = format!("{}", ident);
    let build_struct = quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident {}
    };
    // trait
    let block = &method.block;
    #[cfg(not(feature = "event_args"))]
    let build_trait = quote! {
        #[::proc_qq::re_exports::async_trait::async_trait]
        impl #trait_name for #ident {
            async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
        }
    };
    #[cfg(feature = "event_args")]
    let build_trait = if all_filter_without_bot_command.is_empty() && bot_args.is_none() {
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
        let args_vec = args_to_token(all_filter_without_bot_command);
        if bot_args.is_none() {
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
            for x in bot_args.unwrap() {
                match x {
                    BotParamsMather::Command(command) => {
                        gets.append_all(quote! {
                            if !matcher.match_command(#command) {
                                return Ok(false);
                            }
                        });
                    }
                    BotParamsMather::Params(pat, ty) => {
                        p_pats.append_all(quote! {
                           #pat,
                        });
                        command_params_in_raw.append_all(quote! {
                           #pat: #ty,
                        });
                        gets.append_all(quote! {
                            let #pat: #ty = match ::proc_qq::matcher_get::<#ty>(&mut matcher) {
                                Some(value) => value,
                                None => return Ok(false),
                            };
                        });
                    }
                    BotParamsMather::Multiple(multiple) => {
                        let mut mme = quote! {};
                        let mut pp = vec![];
                        for x in &multiple {
                            match x {
                                BotParamsMatherTuple::Command(name) => {
                                    mme.append_all(quote! {
                                        ::proc_qq::TupleMatcherElement::Command(#name),
                                    });
                                }
                                BotParamsMatherTuple::Params(p, t) => {
                                    mme.append_all(quote! {
                                        ::proc_qq::TupleMatcherElement::Param,
                                    });
                                    pp.push((*p, *t));
                                }
                            }
                        }
                        gets.append_all(quote! {
                            let mut ps = if let Some(ps) = matcher.tuple_matcher(vec![#mme]) {
                                ps
                            } else {
                                return Ok(false);
                            };
                            ps.reverse();
                        });
                        let len = pp.len();
                        gets.append_all(quote! {
                            if ps.len() != #len {
                                return Ok(false);
                            }
                        });
                        for (pat, ty) in pp {
                            p_pats.append_all(quote! {
                              #pat,
                            });
                            command_params_in_raw.append_all(quote! {
                                #pat: #ty,
                            });
                            gets.append_all(quote! {
                                    let #pat: #ty = if let Some(np) = ps.pop() {
                                        let sub_matcher = ::proc_qq::TupleMatcher::new(np);
                                        match ::proc_qq::tuple_matcher_get::<#ty>(sub_matcher) {
                                            Some(value) => value,
                                            None => return Ok(false),
                                        }
                                    } else {
                                        return Ok(false);
                                    };
                            });
                        }
                    }
                }
            }
            quote! {
                #[::proc_qq::re_exports::async_trait::async_trait]
                impl #trait_name for #ident {
                    async fn handle(&self, #param_pat: #param_ty) -> ::proc_qq::re_exports::anyhow::Result<bool> {
                        if !::proc_qq::match_event_args_all(#args_vec, #param_pat.into())? {
                            return Ok(false);
                        }
                        // 匹配指令是否能对应
                        use ::proc_qq::MessageChainPointTrait;
                        let m_chan = #param_pat.message_chain().clone();
                        let mut m_vec = vec![];
                        for x in m_chan {
                            m_vec.push(x);
                        }
                        let mut matcher = ::proc_qq::CommandMatcher::new(m_vec);
                        #gets
                        if matcher.not_blank() {
                            return Ok(false);
                        }
                        self.raw(#param_pat, #p_pats).await
                    }
                }
                impl #ident {
                    async fn raw(&self, #param_pat: #param_ty, #command_params_in_raw) -> ::proc_qq::re_exports::anyhow::Result<bool> #block
                }
            }
        }
    };
    // into
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

fn struct_name(
    pt: &PatType,
    param_ty: String,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match param_ty.as_str() {
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
        "& GroupPoke" => (
            quote! {::proc_qq::GroupPokeProcess},
            quote! {::proc_qq::ModuleEventProcess::GroupPoke},
        ),
        t => abort!(
            pt.span(),
            format!("未知的参数类型 {}, 事件必须作为&self下一个参数(或第一个参数), 请在文档中查看兼容的事件以及参数类型 https://github.com/niuhuan/rust_proc_qq", t),
        ),
    }
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
