use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Block};

#[proc_macro]
pub fn with_block(input: TokenStream) -> TokenStream {
    // Debug the input
    let input_str = input.to_string();
    eprintln!("Input: {}", input_str);

    // Collect tokens into a vector to process
    let input_tokens: Vec<_> = input.into_iter().collect();
    if input_tokens.is_empty() {
        panic!("Expected function or method call followed by a closure block");
    }

    // Find the start of the closure block (last top-level `{ ... }`)
    let mut closure_start = None;
    for (i, token) in input_tokens.iter().enumerate().rev() {
        match token {
            proc_macro::TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace => {
                closure_start = Some(i);
                break;
            }
            _ => {}
        }
    }
    let closure_start = closure_start.expect("No closure block found in macro input");

    // Split tokens into call part and closure part
    let call_tokens: TokenStream = input_tokens[..closure_start]
        .iter()
        .cloned()
        .collect();
    let closure_tokens: TokenStream = input_tokens[closure_start..]
        .iter()
        .cloned()
        .collect();
    
    // Convert to proc_macro2 tokens for better error handling
    let call_tokens_pm2 = proc_macro2::TokenStream::from(call_tokens);
    let closure_tokens_pm2 = proc_macro2::TokenStream::from(closure_tokens);
    
    eprintln!("Call tokens: {}", call_tokens_pm2);
    eprintln!("Closure tokens: {}", closure_tokens_pm2);
    
    // Parse the closure block
    let closure_block: Block = match syn::parse2(closure_tokens_pm2.clone()) {
        Ok(block) => block,
        Err(err) => {
            panic!("Failed to parse closure block: {}\nTokens: {}", err, closure_tokens_pm2);
        }
    };
    
    // Parse the call part
    let call_expr: Expr = match syn::parse2(call_tokens_pm2.clone()) {
        Ok(expr) => expr,
        Err(err) => {
            panic!("Failed to parse call expression: {}\nTokens: {}", err, call_tokens_pm2);
        }
    };
    
    // Generate the final code
    match call_expr {
        Expr::Call(expr_call) => {
            let func = expr_call.func;
            let args = expr_call.args;
            
            // Handle arguments differently to avoid the repetition issue
            let result = if args.is_empty() {
                quote! {
                    #func(|| #closure_block)
                }
            } else {
                // Create a new token stream with the arguments followed by the closure
                let mut all_args = proc_macro2::TokenStream::new();
                for (i, arg) in args.iter().enumerate() {
                    all_args.extend(quote!(#arg));
                    if i < args.len() - 1 {
                        all_args.extend(quote!(,));
                    }
                }
                all_args.extend(quote!(, || #closure_block));
                
                quote! {
                    #func(#all_args)
                }
            };
            
            eprintln!("Result: {}", result);
            result.into()
        }
        Expr::MethodCall(expr_method_call) => {
            let method = expr_method_call.method;
            let receiver = expr_method_call.receiver;
            let args = expr_method_call.args;
            
            // Handle arguments differently to avoid the repetition issue
            let result = if args.is_empty() {
                quote! {
                    #receiver.#method(|| #closure_block)
                }
            } else {
                // Create a new token stream with the arguments followed by the closure
                let mut all_args = proc_macro2::TokenStream::new();
                for (i, arg) in args.iter().enumerate() {
                    all_args.extend(quote!(#arg));
                    if i < args.len() - 1 {
                        all_args.extend(quote!(,));
                    }
                }
                all_args.extend(quote!(, || #closure_block));
                
                quote! {
                    #receiver.#method(#all_args)
                }
            };
            
            eprintln!("Result: {}", result);
            result.into()
        }
        _ => panic!("Expected a function call or method call"),
    }
}