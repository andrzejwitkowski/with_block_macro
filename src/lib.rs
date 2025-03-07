use proc_macro::TokenStream;
use quote::quote;
use syn::{parse2, Expr, Block, parse_macro_input, parse_quote, Pat, Token, parse::Parse, parse::ParseStream, punctuated::Punctuated, Ident};
use proc_macro2::TokenStream as TokenStream2;

// We'll simplify our approach and just pass the raw parameter tokens
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

    // Find the start of the closure block and check for parameters
    let mut closure_start = None;
    let mut params_start = None;
    let mut brace_level = 0;
    
    for (i, token) in input_tokens.iter().enumerate().rev() {
        match token {
            proc_macro::TokenTree::Group(g) if g.delimiter() == proc_macro::Delimiter::Brace => {
                if brace_level == 0 {
                    closure_start = Some(i);
                }
                brace_level += 1;
            },
            proc_macro::TokenTree::Punct(p) if p.as_char() == '>' && brace_level == 0 => {
                // This might be the end of a parameter list like (x, y) ->
                params_start = Some(i - 1); // Position before the arrow
                break;
            },
            _ => {}
        }
    }
    
    let closure_start = closure_start.expect("No closure block found in macro input");
    
    // Split tokens into call part, params part (if any), and closure part
    let (call_tokens, params_tokens): (TokenStream, Option<TokenStream2>) = if let Some(params_pos) = params_start {
        // Find the start of parameters (opening parenthesis)
        let mut param_open = None;
        for i in (0..params_pos).rev() {
            if let proc_macro::TokenTree::Group(g) = &input_tokens[i] {
                if g.delimiter() == proc_macro::Delimiter::Parenthesis {
                    param_open = Some(i);
                    break;
                }
            }
        }
        
        if let Some(param_open) = param_open {
            // For the call part, we need everything before the parameter list
            let call_part: TokenStream = input_tokens[..param_open].iter().cloned().collect();
            
            // Extract parameters - just keep them as a token stream
            let params_part: TokenStream = input_tokens[param_open..=param_open].iter().cloned().collect();
            let params_pm2 = proc_macro2::TokenStream::from(params_part);
            
            (call_part, Some(params_pm2))
        } else {
            let call_part: TokenStream = input_tokens[..closure_start].iter().cloned().collect();
            (call_part, None)
        }
    } else {
        let call_part: TokenStream = input_tokens[..closure_start].iter().cloned().collect();
        (call_part, None)
    };
    
    let closure_tokens: TokenStream = input_tokens[closure_start..].iter().cloned().collect();
    
    // Convert to proc_macro2 tokens for better error handling
    let call_tokens_pm2 = proc_macro2::TokenStream::from(call_tokens);
    let closure_tokens_pm2 = proc_macro2::TokenStream::from(closure_tokens);
    
    // In the closure block parsing section, we need to look for and extract any closure parameters
    
    // Parse the closure block
    let closure_block: Block = match syn::parse2(closure_tokens_pm2.clone()) {
        Ok(block) => block,
        Err(err) => {
            panic!("Failed to parse closure block: {}\nTokens: {}", err, closure_tokens_pm2);
        }
    };
    
    // Extract closure parameters if present
    let mut has_closure_params = false;
    let mut closure_param_tokens = proc_macro2::TokenStream::new();
    let mut modified_stmts = Vec::new();
    
    // Check if the first statement contains a closure parameter declaration
    if let Some(first_stmt) = closure_block.stmts.first() {
        let stmt_str = quote!(#first_stmt).to_string();
        if stmt_str.trim().starts_with('|') {
            has_closure_params = true;
            
            // Extract everything between the first | and the second |
            if let Some(start_idx) = stmt_str.find('|') {
                if let Some(end_idx) = stmt_str[start_idx + 1..].find('|') {
                    let params = &stmt_str[..start_idx + end_idx + 2]; // Include both pipes
                    closure_param_tokens = params.parse::<proc_macro2::TokenStream>().unwrap_or_else(|_| {
                        panic!("Failed to parse closure parameters: {}", params);
                    });
                    
                    // Extract the rest of the statement after the parameter declaration
                    let rest_of_stmt = &stmt_str[start_idx + end_idx + 2..];
                    if !rest_of_stmt.trim().is_empty() {
                        // If there's code on the same line after the params, parse it
                        if let Ok(expr) = syn::parse_str::<syn::Expr>(rest_of_stmt) {
                            modified_stmts.push(syn::Stmt::Expr(expr, None));
                        }
                    }
                    
                    // Add the rest of the statements
                    if closure_block.stmts.len() > 1 {
                        modified_stmts.extend(closure_block.stmts[1..].iter().cloned());
                    }
                }
            }
        } else {
            // No closure parameters, keep all statements
            modified_stmts = closure_block.stmts.clone();
        }
    } else {
        // Empty block
        modified_stmts = Vec::new();
    }
    
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
            
            let result = if args.is_empty() {
                if has_closure_params {
                    quote! {
                        #func(#closure_param_tokens {
                            #(#modified_stmts)*
                        })
                    }
                } else {
                    quote! {
                        #func(|| {
                            #(#modified_stmts)*
                        })
                    }
                }
            } else {
                let mut all_args = proc_macro2::TokenStream::new();
                for (i, arg) in args.iter().enumerate() {
                    all_args.extend(quote!(#arg));
                    if i < args.len() - 1 {
                        all_args.extend(quote!(,));
                    }
                }
                
                if has_closure_params {
                    all_args.extend(quote!(, #closure_param_tokens {
                        #(#modified_stmts)*
                    }));
                } else {
                    all_args.extend(quote!(, || {
                        #(#modified_stmts)*
                    }));
                }
                
                quote! {
                    #func(#all_args)
                }
            };
            
            eprintln!("Result: {}", result);
            result.into()
        }
        Expr::MethodCall(expr_method_call) => {
            // Similar implementation for method calls
            let method = expr_method_call.method;
            let receiver = expr_method_call.receiver;
            let args = expr_method_call.args;
            
            let result = if args.is_empty() {
                if has_closure_params {
                    quote! {
                        #receiver.#method(#closure_param_tokens {
                            #(#modified_stmts)*
                        })
                    }
                } else {
                    quote! {
                        #receiver.#method(|| {
                            #(#modified_stmts)*
                        })
                    }
                }
            } else {
                let mut all_args = proc_macro2::TokenStream::new();
                for (i, arg) in args.iter().enumerate() {
                    all_args.extend(quote!(#arg));
                    if i < args.len() - 1 {
                        all_args.extend(quote!(,));
                    }
                }
                
                if has_closure_params {
                    all_args.extend(quote!(, #closure_param_tokens {
                        #(#modified_stmts)*
                    }));
                } else {
                    all_args.extend(quote!(, || {
                        #(#modified_stmts)*
                    }));
                }
                
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