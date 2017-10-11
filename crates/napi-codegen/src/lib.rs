#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private, quote)]

extern crate syntax;
extern crate rustc;
extern crate rustc_plugin;

use rustc_plugin::Registry;
use syntax::ast::{FnDecl, Ident, ItemKind, LitKind, MetaItem};
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::codemap::Span;
use syntax::symbol::Symbol;

fn get_js_name(ecx: &mut ExtCtxt, span: Span, meta_item: &MetaItem) -> String {
    let meta_items = meta_item.meta_item_list().unwrap_or_else(|| {
        ecx.struct_span_err(span, "incorrect use of napi_callback attribute")
            .help("usage: #[napi_callback(\"jsFunctionName\")]")
            .emit();
        ecx.span_fatal(span, "invalid attribute usage");
    });

    if meta_items.len() != 1 {
        ecx.struct_span_err(span, "incorrect use of napi_callback attribute")
            .help("usage: #[napi_callback(\"jsFunctionName\")]")
            .emit();
        ecx.span_fatal(span, "attribute requires exactly one argument");
    }

    let name_meta_item = &meta_items[0];

    let name = name_meta_item
        .literal()
        .and_then(|literal| match literal.node {
            LitKind::Str(ref s, _) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or_else(|| {
            ecx.span_fatal(
                name_meta_item.span(),
                "attribute argument must be a string",
            );
        });

    String::from(&*name)
}

struct Function {
    pub ident: Ident,
    pub decl: FnDecl,
}

impl Function {
    fn from_annotatable(
        ecx: &mut ExtCtxt,
        annotated: &Annotatable,
    ) -> Function {
        let report_error = |span| {
            ecx.span_fatal(
                span,
                "napi_callback attribute can only be used on \
                functions, but was applied to this item.",
            );
        };

        match *annotated {
            Annotatable::Item(ref item) => {
                match item.node {
                    ItemKind::Fn(ref decl, ..) => Function {
                        ident: item.ident,
                        decl: decl.clone().unwrap(),
                    },
                    _ => report_error(item.span),
                }
            }
            Annotatable::TraitItem(ref item) => report_error(item.span),
            Annotatable::ImplItem(ref item) => report_error(item.span),
        }
    }
}

fn create_napi_callback(ecx: &mut ExtCtxt, function: &Function) -> Annotatable {
    let fn_ident = function.ident;
    let extern_fn_name = format!("napi_rs_cb_{}", fn_ident.name);
    let extern_fn_ident = Ident::from_str(extern_fn_name.as_str());

    Annotatable::Item(
        quote_item!(ecx,
            #[no_mangle]
            pub extern "C" fn $extern_fn_ident(
                env: ::napi::sys::napi_env,
                _info: ::napi::sys::napi_callback_info,
            ) -> ::napi::sys::napi_value {
                use std::error::Error;
                use std::ffi::CString;
                use std::mem;
                use std::ptr;

                use ::napi::{NapiEnv, NapiResult, NapiValue};
                use ::napi::sys::{napi_get_undefined, napi_throw,
                                  napi_throw_error, napi_value};

                let env_wrapper = NapiEnv::from(env);

                fn typecheck_result<T: NapiValue>(_: &NapiResult<T>) {}

                let result = $fn_ident(&env_wrapper);
                typecheck_result(&result);

                match result {
                    Ok(value) => value.as_sys_value(),
                    Err(error) => {
                        if let Some(exception) = error.exception {
                            unsafe {
                                napi_throw(env, exception);
                            }
                        } else {
                            let message = format!("{}", &error);
                            let c_string =
                                CString::new(message).unwrap_or_else(|_| {
                                    CString::new(error.description()).unwrap()
                                });

                            unsafe {
                                napi_throw_error(
                                    env,
                                    ptr::null(),
                                    c_string.as_ptr(),
                                );
                            }
                        }

                        unsafe {
                            let mut result: napi_value = mem::uninitialized();
                            napi_get_undefined(env, &mut result);
                            result
                        }
                    }
                }
            }
        ).unwrap(),
    )
}

pub fn napi_callback_decorator(
    ecx: &mut ExtCtxt,
    span: Span,
    meta_item: &MetaItem,
    annotated: Annotatable,
) -> Vec<Annotatable> {
    let js_name = get_js_name(ecx, span, meta_item);
    let function = Function::from_annotatable(ecx, &annotated);

    let mut output = Vec::new();

    output.push(create_napi_callback(ecx, &function));
    output.push(annotated);

    output
}

#[plugin_registrar]
pub fn plugin_registrar(registry: &mut Registry) {
    registry.register_syntax_extension(
        Symbol::intern("napi_callback"),
        SyntaxExtension::MultiModifier(Box::new(napi_callback_decorator)),
    );
}
