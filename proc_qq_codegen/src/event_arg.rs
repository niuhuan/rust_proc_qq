use proc_macro_error::abort;
use quote::{quote, TokenStreamExt};
use syn::spanned::Spanned;
use syn::Lit::Str;
use syn::Meta::{List, NameValue, Path};
use syn::NestedMeta;
use syn::NestedMeta::{Lit, Meta};

#[derive(Clone, Debug)]
pub(crate) enum EventArg {
    All(Vec<EventArg>),
    Any(Vec<EventArg>),
    Not(Vec<EventArg>),
    Regexp(String),
    Eq(String),
    TrimRegexp(String),
    TrimEq(String),
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
