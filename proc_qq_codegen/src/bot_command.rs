use std::ops::Deref;

use proc_macro2::Ident;
use proc_macro_error::abort;
use syn::{FnArg, ItemFn, Pat, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BotCommandRaw {
    Command(String),
    Param(String),
    Enum(String, Vec<String>),
    Multiple(Vec<BotCommandRawTuple>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BotCommandRawTuple {
    Command(String),
    Param(String),
    Enum(String, Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BotParamsMather<'a> {
    Command(String),
    Params(&'a Ident, &'a Type),
    Enum(&'a Ident, &'a Type, Vec<String>),
    Multiple(Vec<BotParamsMatherTuple<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BotParamsMatherTuple<'a> {
    Command(String),
    Params(&'a Ident, &'a Type),
    Enum(&'a Ident, &'a Type, Vec<String>),
}

const COMMAND_NOTICE: &str = r#"bot_command中的元素必须是由固定字符串和参数组合而成, 例如 "/删除 {idx}" "请{min}{time:|小时|分钟|秒钟}后提醒我{event}" "{option:|开启|关闭} 天气预报" "#;

// 解析命令行
pub(crate) fn parse_bot_command(
    method: &ItemFn,
    bot_command: Option<String>,
) -> Option<Vec<BotCommandRaw>> {
    // 由固定字符串和参数组合而成
    if let Some(bot_command) = bot_command {
        //
        let command_reg = regex::Regex::new(r#"^[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+$"#)
            .expect("proc_qq正则错误(command_reg)");
        let params_reg = regex::Regex::new(r#"^\{[A-Za-z_]([A-Za-z0-9_]+)?(:(\|[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)+)?\}$"#)
            .expect("proc_qq正则错误(params_reg)");
        let tuple_reg = regex::Regex::new(r#"^(([A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)|(\{[A-Za-z_]([A-Za-z0-9_]+)?(:(\|[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)+)?\}))+$"#)
            .expect("proc_qq正则错误(tuple_reg)");
        //
        let mut bot_command_items = vec![];
        let mut bot_command_item_strs = bot_command.split_whitespace();
        // 根据空格切分并循环
        while let Some(item) = bot_command_item_strs.next() {
            if command_reg.is_match(item) {
                bot_command_items.push(BotCommandRaw::Command(item.to_owned()));
            } else if params_reg.is_match(item) {
                let param = &item[1..item.len() - 1];
                if param.contains(":|") {
                    let idx = param.find(":|").unwrap();
                    let param_name = &param[..idx];
                    let param_enum_str = &param[idx + 2..];
                    let param_enums = param_enum_str.split('|');
                    bot_command_items.push(BotCommandRaw::Enum(
                        param_name.to_string(),
                        param_enums.map(|s| s.to_string()).collect(),
                    ));
                } else {
                    bot_command_items.push(BotCommandRaw::Param(param.to_string()));
                }
            } else if tuple_reg.is_match(item) {
                let mut bot_command_elements = vec![];
                let find_reg = regex::Regex::new(r#"(([A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)|(\{[A-Za-z_]([A-Za-z0-9_]+)?(:(\|[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)+)?\}))"#).expect("proc_qq正则错误(find_reg)");
                for x in find_reg.find_iter(item) {
                    let element_str = &item[x.start()..x.end()];
                    if element_str.starts_with("{") {
                        let param = &element_str[1..element_str.len() - 1];
                        if param.contains(":|") {
                            let idx = param.find(":|").unwrap();
                            let param_name = &param[..idx];
                            let param_enum_str = &param[idx + 2..];
                            let param_enums = param_enum_str.split('|');
                            bot_command_elements.push(BotCommandRawTuple::Enum(
                                param_name.to_string(),
                                param_enums.map(|s| s.to_string()).collect(),
                            ));
                        } else {
                            bot_command_elements.push(BotCommandRawTuple::Param(param.to_string()));
                        }
                    } else {
                        bot_command_elements
                            .push(BotCommandRawTuple::Command(element_str.to_string()));
                    }
                }
                bot_command_items.push(BotCommandRaw::Multiple(bot_command_elements))
            } else {
                abort!(
                    &method.sig.ident.span(),
                    format!("{} => {}", COMMAND_NOTICE, item)
                );
            }
        }
        Some(bot_command_items)
    } else {
        None
    }
}

// 将命令行跟参数进行匹配
pub(crate) fn parse_bot_args<'a>(
    method: &'a ItemFn,
    args: &'a [&'a FnArg],
    items: Option<Vec<BotCommandRaw>>,
) -> Option<Vec<BotParamsMather<'a>>> {
    if let Some(items) = items {
        let mut result = vec![];
        let mut args_iter = args.iter();
        for item in items {
            result.push(match item {
                BotCommandRaw::Command(tmp) => BotParamsMather::Command(tmp),
                BotCommandRaw::Param(tmp) => {
                    let (pat, ty) = take_param(method, args_iter.next(), tmp.as_str());
                    BotParamsMather::Params(pat, ty)
                }
                BotCommandRaw::Enum(tmp, e) => {
                    let (pat, ty) = take_param(method, args_iter.next(), tmp.as_str());
                    BotParamsMather::Enum(pat, ty, e.clone())
                }
                BotCommandRaw::Multiple(multiple) => {
                    let mut multiple_result = vec![];
                    for m in multiple {
                        multiple_result.push(match m {
                            BotCommandRawTuple::Command(tmp) => {
                                BotParamsMatherTuple::Command(tmp.clone())
                            }
                            BotCommandRawTuple::Param(tmp) => {
                                let (pat, ty) = take_param(method, args_iter.next(), tmp.as_str());
                                BotParamsMatherTuple::Params(pat, ty)
                            }
                            BotCommandRawTuple::Enum(tmp, e) => {
                                let (pat, ty) = take_param(method, args_iter.next(), tmp.as_str());
                                BotParamsMatherTuple::Enum(pat, ty, e.clone())
                            }
                        })
                    }
                    BotParamsMather::Multiple(multiple_result)
                }
            });
        }
        Some(result)
    } else {
        None
    }
}

fn take_param<'a>(method: &'a ItemFn, arg: Option<&&'a FnArg>, tmp: &str) -> (&'a Ident, &'a Type) {
    if let Some(arg) = arg {
        match arg {
            FnArg::Receiver(_) => {
                abort!(&method.sig.ident.span(), "bot_command的参数不支持Self");
            }
            FnArg::Typed(t) => {
                let pat = t.pat.deref();
                match pat {
                    Pat::Ident(pi) => {
                        let ident = &pi.ident;
                        if ident.to_string().eq(tmp) {
                            let ty = t.ty.deref();
                            return (ident, ty);
                        } else {
                            abort!(
                                &method.sig.ident.span(),
                                "bot_command中的参数名必须与参数名一致 {} != {}",
                                tmp,
                                pi.ident,
                            );
                        }
                    }
                    _ => {
                        abort!(
                            &method.sig.ident.span(),
                            "bot_command中的参数必须是标识符 {}",
                            tmp,
                        );
                    }
                }
            }
        }
    } else {
        abort!(
            &method.sig.ident.span(),
            "参数个数与bot_command不匹配，您的方法中缺少参数 : {}",
            tmp,
        );
    }
}
