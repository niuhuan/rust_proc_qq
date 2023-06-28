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
        let element_reg_str = r#"([A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)|(\{([A-Za-z_]([A-Za-z0-9_]+)?)(:(|[A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)+)?\})"#;
        let elements_reg = regex::Regex::new(format!("^({})+$", element_reg_str).as_str())
            .expect("bot_command正则表达式编译失败");
        let element_reg =
            regex::Regex::new(element_reg_str).expect("bot_command正则表达式编译失败");
        let mut bot_command_items = vec![];
        let mut bot_command_item_strs = bot_command.split_whitespace();
        // 根据空格切分并循环
        while let Some(item) = bot_command_item_strs.next() {
            // 不符合正则表达式将会提示错误
            if !elements_reg.is_match(item) {
                abort!(&method.sig.ident.span(), COMMAND_NOTICE);
            }
            let mut element_strs = element_reg.find_iter(item);
            let mut bot_command_elements = vec![];
            // 多次匹配
            while let Some(element_str) = element_strs.next() {
                let element_str = element_str.as_str();
                if element_str.starts_with('{') {
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
                    bot_command_elements.push(BotCommandRawTuple::Command(element_str.to_string()));
                }
            }
            if bot_command_elements.is_empty() {
                // PROC_QQ逻辑错误才会运行此分支
                abort!(&method.sig.ident.span(), COMMAND_NOTICE);
            }
            if bot_command_elements.len() == 1 {
                bot_command_items.push(match bot_command_elements.first().unwrap() {
                    BotCommandRawTuple::Command(tmp) => BotCommandRaw::Command(tmp.clone()),
                    BotCommandRawTuple::Param(tmp) => BotCommandRaw::Param(tmp.clone()),
                    BotCommandRawTuple::Enum(n, e) => BotCommandRaw::Enum(n.clone(), e.clone()),
                });
            } else {
                bot_command_items.push(BotCommandRaw::Multiple(bot_command_elements))
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
