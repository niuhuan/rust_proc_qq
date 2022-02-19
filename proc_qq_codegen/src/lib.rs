use proc_macro::TokenStream;

use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, FnArg};

use proc_macro_error::{abort, proc_macro_error};

macro_rules! emit {
    ($tokens:expr) => {{
        use devise::ext::SpanDiagnosticExt;

        let mut tokens = $tokens;
        if std::env::var_os("PROC_QQ_CODEGEN_DEBUG").is_some() {
            let debug_tokens = proc_macro2::Span::call_site()
                .note("emitting proc qq codegen debug output")
                .note(tokens.to_string())
                .emit_as_item_tokens();

            tokens.extend(debug_tokens);
        }

        tokens.into()
    }};
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn event(_: TokenStream, input: TokenStream) -> TokenStream {
    // 必须加在方法上
    let method = parse_macro_input!(input as syn::ItemFn);
    // 必须是异步方法
    if method.sig.asyncness.is_none() {
        abort!(&method.sig.span(), "必须是async方法");
    }
    // visible
    let _ = &method.vis;
    // params
    let params = &method.sig.inputs;
    if params.len() != 1 {
        abort!(&method.sig.span(), "必须有且只能有一个参数");
    };
    let param = params.first().unwrap();
    let param = match param {
        FnArg::Receiver(_) => abort!(&param.span(), "不支持self"),
        FnArg::Typed(pt) => pt,
    };

    //let param_pat = param.pat.as_ref();
    let param_ty = param.ty.as_ref();
    //let param_pat = quote! {#param_pat};
    let param_ty = quote! {#param_ty};
    if !vec!["& GroupMessageEvent"].contains(&param_ty.to_string().as_ref()) {
        abort!(param.span(), "param type must be &GroupMessageEvent");
    }

    let finish = quote! {
        #method

    };
    //abort!(&method.sig.span(), format!("{}", param_ty));
    emit!(finish)
}
