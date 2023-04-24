use proc_macro_error::abort;
use syn::{FnArg, ItemFn};

pub(crate) type BotCommandItem = Vec<BotCommandItemElement>;

pub(crate) enum BotCommandItemElement {
    Command(String),
    Param(String),
}

pub(crate) fn parse_bot_command(
    method: &ItemFn,
    bot_command: Option<String>,
) -> Option<Vec<BotCommandItem>> {
    // 由固定字符串和参数组合而成
    let element_reg_str =
        r#"([A-Za-z0-9_/\p{Han}\p{Hiragana}\p{Katakana}]+)|(\{([A-Za-z_]([A-Za-z0-9_]+)?)\})"#;
    let elements_reg = regex::Regex::new(format!("^({})+$", element_reg_str).as_str())
        .expect("bot_command正则表达式编译失败");
    let element_reg = regex::Regex::new(element_reg_str).expect("bot_command正则表达式编译失败");
    if let Some(bot_command) = bot_command {
        let mut bot_command_element_strs = bot_command.split_whitespace();
        while let Some(element_str) = bot_command_element_strs.next() {
            if !elements_reg.is_match(element_str) {
                abort!(
                    &method.sig.span(),
                    r#"bot_command中的元素必须是由固定字符串和参数组合而成 "/删除 {idx}" "请{min}分钟后提醒我{event}" "#
                );
            }
            let mut element_strs = element_reg.find_iter(element_str);
            let mut bot_command_item = vec![];
            while let Some(element_str) = element_strs.next() {
                let element_str = element_str.as_str();
                if element_str.starts_with('{') {
                    bot_command_item.push(BotCommandItemElement::Param(
                        element_str[1..element_str.len() - 1].to_string(),
                    ));
                } else {
                    bot_command_item.push(BotCommandItemElement::Command(element_str.to_string()));
                }
            }
            if bot_command_item.is_empty() {
                abort!(
                    &method.sig.span(),
                    "bot_command中的元素必须是由固定字符串和参数组合而成"
                );
            }
            bot_command_items.push(bot_command_item);
        }
        Some(result)
    } else {
        None
    }
}

pub enum ParamsMather<'a> {
    Command(String),
    Params(&'a syn::Pat, &'a syn::Type),
    Boundary,
}

pub(crate) fn parse_bot_args(
    method: &ItemFn,
    args: &[&FnArg],
    items: Option<Vec<BotCommandItem>>,
) -> Option<Vec<ParamsMather>> {
    if let Some(items) = items {
        let mut result = vec![];
        let mut args = args.iter();
        for item in items {
            for item_element in item {
                match item_element {
                    BotCommandItemElement::Command(command) => {
                        result.push(ParamsMather::Command(command));
                    }
                    BotCommandItemElement::Param(param) => {
                        if let Some(arg) = args.next() {
                            if let FnArg::Typed(arg) = arg {
                                if let syn::Pat::Ident(arg) = &*arg.pat {
                                    if arg.ident.to_string().eq(param.as_str()) {
                                        result.push(ParamsMather::Params(arg.pat, arg.ty));
                                    } else {
                                        abort!(
                                            &method.sig.span(),
                                            "bot_command中的参数名必须与参数名一致 {} != {}",
                                            param,
                                            arg.ident,
                                        );
                                    }
                                } else {
                                    abort!(&method.sig.span(), "bot_command的参数不支持Self");
                                }
                            } else {
                                abort!(&method.sig.span(), "bot_command中的参数名必须与参数名一致");
                            }
                        } else {
                            abort!(
                                &method.sig.span(),
                                "参数个数与bot_command不匹配，您的方法中缺少参数"
                            );
                        }
                    }
                }
            }
            result.push(ParamsMather::Boundary);
        }
        if let Some(_) = args.next() {
            abort!(
                &method.sig.span(),
                "参数个数与bot_command不匹配, 您的方法中有多余的参数"
            );
        }
    }
    if args.len() > 0 {
        abort!(
            &method.sig.span(),
            "您未指定bot_command, 不能使用更多的参数"
        );
    }
    None
}
