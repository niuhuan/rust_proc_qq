use proc_macro_error::abort;
use quote::{quote, TokenStreamExt};
use syn::spanned::Spanned;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta::{Lit, Meta};
use syn::{AttributeArgs, ItemFn, NestedMeta};

#[derive(Clone, Debug)]
pub(crate) enum EventArg {
    All(Vec<EventArg>),
    Any(Vec<EventArg>),
    Not(Vec<EventArg>),
    Regexp(String),
    Eq(String),
    TrimRegexp(String),
    TrimEq(String),
    BotCommand(String),
}

// 递归匹配表达式
pub(crate) fn parse_args(children: Vec<NestedMeta>) -> Vec<EventArg> {
    let mut children_args = vec![];
    for nm in children {
        match nm {
            Meta(meta) => match meta {
                Path(_) => abort!(&meta.span(), "不支持的参数名称"),
                List(list) => {
                    if list.path.segments.len() != 1 {
                        abort!(&list.span(), "表达式有且只能有一个片段");
                    }
                    let ident = &list.path.segments.first().unwrap().ident;
                    let ident_name = list.path.segments.first().unwrap().ident.to_string();
                    match ident_name.as_str() {
                        "all" => {
                            let mut v = vec![];
                            for x in list.nested {
                                v.push(x);
                            }
                            children_args.push(EventArg::All(parse_args(v)));
                        }
                        "not" => {
                            let mut v = vec![];
                            for x in list.nested {
                                v.push(x);
                            }
                            children_args.push(EventArg::Not(parse_args(v)));
                        }
                        "any" => {
                            let mut v = vec![];
                            for x in list.nested {
                                v.push(x);
                            }
                            children_args.push(EventArg::Any(parse_args(v)));
                        }
                        _ => abort!(&ident.span(), "不支持的参数名称"),
                    }
                }
                NameValue(nv) => {
                    if nv.path.segments.len() != 1 {
                        abort!(&nv.path.span(), "表达式有且只能有一个片段");
                    }
                    let ident = &nv.path.segments.first().unwrap().ident;
                    let ident_name = nv.path.segments.first().unwrap().ident.to_string();
                    match ident_name.as_str() {
                        "regexp" => match nv.lit {
                            Str(value) => {
                                let v = value.value();
                                match regex::Regex::new(v.as_str()) {
                                    Ok(_) => {
                                        children_args.push(EventArg::Regexp(v));
                                    }
                                    Err(_) => {
                                        abort!(&ident.span(), "正则表达式不正确");
                                    }
                                }
                            }
                            _ => abort!(&ident.span(), "regexp只支持字符串类型参数值"),
                        },
                        "eq" => match nv.lit {
                            Str(value) => {
                                children_args.push(EventArg::Eq(value.value()));
                            }
                            _ => abort!(&ident.span(), "eq只支持字符串类型参数值"),
                        },
                        "trim_regexp" => match nv.lit {
                            Str(value) => {
                                let v = value.value();
                                match regex::Regex::new(v.as_str()) {
                                    Ok(_) => {
                                        children_args.push(EventArg::TrimRegexp(v));
                                    }
                                    Err(_) => {
                                        abort!(&ident.span(), "正则表达式不正确");
                                    }
                                }
                            }
                            _ => abort!(&ident.span(), "trim_regexp只支持字符串类型参数值"),
                        },
                        "trim_eq" => match nv.lit {
                            Str(value) => {
                                children_args.push(EventArg::TrimEq(value.value()));
                            }
                            _ => abort!(&ident.span(), "trim_eq只支持字符串类型参数值"),
                        },
                        "bot_command" => match nv.lit {
                            Str(value) => {
                                children_args.push(EventArg::BotCommand(value.value()));
                            }
                            _ => abort!(&ident.span(), "bot_command只支持字符串类型参数值"),
                        },
                        _ => abort!(&ident.span(), "不支持的参数名称"),
                    }
                }
            },
            Lit(ident) => abort!(&ident.span(), "不支持值类型的参数"),
        }
    }
    children_args
}

pub(crate) fn arg_to_token(arg: EventArg) -> proc_macro2::TokenStream {
    match arg {
        EventArg::All(v) => {
            let ts = args_to_token(v);
            quote! {
                ::proc_qq::EventArg::All(#ts)
            }
        }
        EventArg::Any(v) => {
            let ts = args_to_token(v);
            quote! {
                ::proc_qq::EventArg::Any(#ts)
            }
        }
        EventArg::Not(v) => {
            let ts = args_to_token(v);
            quote! {
                ::proc_qq::EventArg::Not(#ts)
            }
        }
        EventArg::Eq(string) => {
            quote! {
                ::proc_qq::EventArg::Eq(#string .to_string())
            }
        }
        EventArg::Regexp(string) => {
            quote! {
                ::proc_qq::EventArg::Regexp(#string .to_string())
            }
        }
        EventArg::TrimEq(string) => {
            quote! {
                ::proc_qq::EventArg::TrimEq(#string .to_string())
            }
        }
        EventArg::TrimRegexp(string) => {
            quote! {
                ::proc_qq::EventArg::TrimRegexp(#string .to_string())
            }
        }
        EventArg::BotCommand(_) => {
            panic!("BotCommand 不能被序列化")
        }
    }
}

pub(crate) fn args_to_token(all: Vec<EventArg>) -> proc_macro2::TokenStream {
    let mut args = quote! {};
    for arg in all {
        args.append_all(arg_to_token(arg));
        args.append_all(quote! {,})
    }
    quote! {
        vec![#args]
    }
}

pub(crate) fn contains_bot_command(all: &Vec<EventArg>) -> bool {
    for x in all {
        match x {
            EventArg::BotCommand(_) => {
                return true;
            }
            EventArg::All(args) => {
                if contains_bot_command(args) {
                    return true;
                }
            }
            EventArg::Any(args) => {
                if contains_bot_command(args) {
                    return true;
                }
            }
            EventArg::Not(args) => {
                if contains_bot_command(args) {
                    return true;
                }
            }
            EventArg::Regexp(_) => {}
            EventArg::Eq(_) => {}
            EventArg::TrimRegexp(_) => {}
            EventArg::TrimEq(_) => {}
        }
    }
    false
}

pub(crate) fn parse_args_and_command(
    method: &ItemFn,
    attrs: AttributeArgs,
) -> (Vec<EventArg>, Option<String>) {
    // 从众多EventArg中找到bot_command（如果存在）
    let all: Vec<EventArg> = parse_args(attrs);
    let mut bot_command = None;
    let mut _all = vec![];
    for x in all {
        if let EventArg::BotCommand(command) = x {
            if bot_command.is_none() {
                bot_command = Some(command);
            } else {
                abort!(
                    &method.sig.span(),
                    "bot_command 只能有一个，且必须是直接写在event括号中"
                );
            }
        } else {
            _all.push(x);
        }
    }
    let all = _all;
    if contains_bot_command(&all) {
        // 这里是为了判断all/in之类的聚合指令内部有没有bot_command，在其指令内部包括bot_command不被允许。因为场景太少，而且逻辑复杂入不敷出。
        abort!(
            &method.sig.span(),
            "bot_command 只能有一个，且必须是直接写在event括号中"
        );
    }
    (all, bot_command)
}
