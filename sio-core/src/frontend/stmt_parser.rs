use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use crate::alloc::string::ToString;
use crate::frontend::{
    ast::*,
    token::*,
    common::*,
    parser::Parser,
    position::Span,
    position::WithSpan,
};

fn parse_program(it: &mut Parser) -> Result<Vec<WithSpan<Stmt>>, ()> {
    let mut statements = Vec::new();
    while !it.is_eof() {
        statements.push(parse_module_declaration(it)?);
    }

    Ok(statements)
}

fn parse_module_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    match it.peek() {
        TokenKind::Url => parse_url_declaration(it),
        TokenKind::Corporal => parse_corporal_declaration(it),
        TokenKind::Major => parse_major_declaration(it),
        TokenKind::Brigadier => parse_brigadier_declaration(it),
        TokenKind::General => parse_general_declaration(it),
        _ => {
            it.error(&format!("Unexpected {}", it.peek_token().value), it.peek_token().span);
            Err(())
        },
    }
}
fn parse_url_declaration(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = p.expect(TokenKind::Url)?;
    let name = expect_identifier(p)?;
    p.expect(TokenKind::Colon)?;
    let hierarchical_names = parse_hierarchical_components(p)?;
    let end_span = p.expect(TokenKind::Semicolon)?;

    Ok(WithSpan::new(
        Stmt::Url(Box::new(name), hierarchical_names),
        Span::union(begin_span, end_span),
    ))
}

fn parse_hierarchical_components(p: &mut Parser) -> Result<Vec<WithSpan<UrlComponent>>, ()> {
    let mut components = Vec::new();

    // Expect the first component of the URL
    match p.peek() {
        TokenKind::String => {
            let string = p.expect(TokenKind::String)?;
            let string_value = WithSpan::new(string.value.to_string(), string.span);
            components.push(WithSpan::new(UrlComponent::String(string_value), string.span));
        }
        TokenKind::Identifier => {
            let identifier = p.expect(TokenKind::Identifier)?;
            let identifier_value = WithSpan::new(identifier.value.to_string(), identifier.span);
            components.push(WithSpan::new(UrlComponent::Identifier(identifier_value), identifier.span));
        }
        _ => return Err(()),
    }

    while p.peek() == TokenKind::ColonColon {
        p.expect(TokenKind::ColonColon)?;
        match p.peek() {
            TokenKind::String => {
                let string = p.expect(TokenKind::String)?;
                let string_value = WithSpan::new(string.value.to_string(), string.span);
                components.push(WithSpan::new(UrlComponent::String(string_value), string.span));
            }
            TokenKind::Identifier => {
                let identifier = p.expect(TokenKind::Identifier)?;
                let identifier_value = WithSpan::new(identifier.value.to_string(), identifier.span);
                components.push(WithSpan::new(UrlComponent::Identifier(identifier_value), identifier.span));
            }
            _ => return Err(()),
        }
    }

    Ok(components)
}

fn parse_module<F>( p: &mut Parser, token_kind: TokenKind, create_module: F) -> Result<WithSpan<Stmt>, ()>
where
    F: FnOnce(Vec<WithSpan<UrlComponent>>, Vec<WithSpan<Stmt>>) -> Module,
{
    let begin_span = p.expect(token_kind)?;
    let name = parse_hierarchical_components(p)?;
    p.expect(TokenKind::LeftBrace)?;
    let statements = parse_module_declarations(p)?;
    let end_span = p.expect(TokenKind::RightBrace)?;
    Ok(WithSpan::new(
        Stmt::Module(create_module(name, statements)), 
        Span::union(&begin_span, &end_span),
    ))
}
fn parse_general_declaration(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let general_module = | name: Vec<WithSpan<UrlComponent>>, stmts: Vec<WithSpan<Stmt>> | { 
        Module::General{ name, stmts }
    };
    parse_module(p, TokenKind::General, general_module)
}
fn parse_brigadier_declaration(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let brigadier_module = | name: Vec<WithSpan<UrlComponent>>, stmts: Vec<WithSpan<Stmt>> | { 
        Module::Brigadier{ name, stmts }
    };
    parse_module(p, TokenKind::Brigadier, brigadier_module)
}

fn parse_major_declaration(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let major_module = | name: Vec<WithSpan<UrlComponent>>, stmts: Vec<WithSpan<Stmt>> | { 
        Module::Major{ name, stmts }
    };
    parse_module(p, TokenKind::Major, major_module)
}

fn parse_corporal_declaration(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let corporal_module = | name: Vec<WithSpan<UrlComponent>>, stmts: Vec<WithSpan<Stmt>> | { 
        Module::Corporal{ name, stmts }
    };
    parse_module(p, TokenKind::Corporal, corporal_module)
}

fn parse_module_declarations(p: &mut Parser) -> Result<Vec<WithSpan<Stmt>>, ()> {
    let mut statements: Vec<WithSpan<Stmt>> = Vec::new();
    while !p.check(TokenKind::RightBrace) && !p.is_eof() {
        match p.peek() {
            TokenKind::Url => {
                statements.push(parse_url_declaration(p)?);
            }
            TokenKind::Use => statements.push(parse_use_statement(p)?),
            TokenKind::Pub | TokenKind::Identifier => {
                statements.push(parse_function_declaration(p)?);
            }
            _ => {
                let token = p.advance();
                p.error(&format!("Unexpected {}", token.value), token.span);
                return Err(());
            }
        }
    }
    Ok(statements)
}

fn parse_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    match it.peek() {
        TokenKind::Url => parse_url_declaration(it),
        TokenKind::Identifier => {
            let next_token = it.peek_next();
            match next_token {
                TokenKind::ColonColon => parse_function_declaration(it),
                _ => parse_statement(it),
            }
        },
        _ => parse_statement(it),
    }
}


pub fn parse_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    match it.peek() {
        TokenKind::Print => parse_print_statement(it),
        TokenKind::If => parse_if_statement(it),
        TokenKind::LeftBrace => parse_block_statement(it),
        TokenKind::Use => parse_use_statement(it),
        TokenKind::Let => parse_let_statement(it),
        TokenKind::Thread => parse_thread_statement(it),
        _ => parse_expr_statement(it),
    }
}

fn parse_use_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Use)?;
    let path = parse_use_path(it)?;
    let items = if it.check(TokenKind::LeftBrace) {
        it.expect(TokenKind::LeftBrace)?;
        let sub_items = parse_use_items(it)?;
        it.expect(TokenKind::RightBrace)?;
        sub_items
    } else {
        vec![UseItem::Simple { path: path.clone() }]
    };
    let end_span = it.expect(TokenKind::Semicolon)?;
    let use_stmt = if items.len() == 1 && matches!(items[0], UseItem::Simple { .. }) {
        Stmt::Use(path, vec![])
    } else {
        Stmt::Use(path, items)
    };
    let span = Span::union(&begin_span, &end_span);
    Ok(WithSpan::new(use_stmt, span))
}

fn parse_use_path(it: &mut Parser) -> Result<Vec<WithSpan<String>>, ()> {
    let mut path = Vec::new();
    while it.check(TokenKind::Identifier) {
        path.push(expect_identifier(it)?);
        if it.check(TokenKind::ColonColon) {
            it.expect(TokenKind::ColonColon)?;
        } else {
            break;
        }
    }
    Ok(path)
}

fn parse_use_items(it: &mut Parser) -> Result<Vec<UseItem>, ()> {
    let mut items = Vec::new();
    loop {
        if it.check(TokenKind::RightBrace) {
            break;
        }
        let item_path = parse_use_path(it)?;
        if it.check(TokenKind::LeftBrace) {
            it.expect(TokenKind::LeftBrace)?;
            let sub_items = parse_use_items(it)?;
            it.expect(TokenKind::RightBrace)?;
            items.push(UseItem::Nested { path: item_path, items: sub_items });
        } else {
            items.push(UseItem::Simple { path: item_path });
        }
        if it.check(TokenKind::Comma) {
            it.expect(TokenKind::Comma)?;
        } else {
            break;
        }
    }
    Ok(items)
}

fn parse_function_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let visibility = if it.check(TokenKind::Pub) {
        it.expect(TokenKind::Pub)?;
        Visibility::Public
    } else {
        Visibility::Private
    };

    let name = expect_identifier(it)?;

    if !it.check(TokenKind::ColonColon) {
        let token = it.advance();
        it.error(&format!("Unexpected {} found. Expected '::' after function name '{}'.", token.value, name.value), token.span);
        return Err(())
    }
    it.expect(TokenKind::ColonColon)?;
    it.expect(TokenKind::LeftParen)?;
    let params = parse_params(it)?;
    it.expect(TokenKind::RightParen)?;

    let block_stmt = parse_block_statement(it)?;

    let function = Function {
        visibility,
        name: Some(name.clone()),
        params,
        body: Box::new(block_stmt.clone()),
    };

    let stmt = Stmt::Function(function);
    let span = Span::union(&name, &block_stmt);
    Ok(WithSpan::new(stmt, span))
}

/*
fn parse_function_declaration(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let visibility = if it.check(TokenKind::Pub) {
        it.expect(TokenKind::Pub)?;
        Visibility::Public
    } else {
        Visibility::Private
    };

    let name = expect_identifier(it)?;
    //ensure '::' is present; otherwise error out. This is a function application
    if !it.check(TokenKind::ColonColon) {
        let token = it.advance();
        it.error(&format!("Unexpected {} found. Expected '::' after function name '{}'.", token.value, name.value), token.span);
        return Err(())
    }
    it.expect(TokenKind::ColonColon)?;
    it.expect(TokenKind::LeftParen)?;
    let params = parse_params(it)?;
    it.expect(TokenKind::RightParen)?;
    let return_type = if it.check(TokenKind::Arrow) {
        it.expect(TokenKind::Arrow)?;
        Some(expect_identifier(it)?)
    } else {
        None
    };

    it.expect(TokenKind::LeftBrace)?;

    let mut body: Vec<WithSpan<Stmt>> = Vec::new();
    while !it.check(TokenKind::RightBrace) && !it.is_eof() {
        body.push(parse_declaration(it)?);
    }

    let end_span = it.expect(TokenKind::RightBrace)?;

    let function = Function {
        visibility,
        name: Some(name.clone()),
        params,
        return_type,
        body,
    };

    let stmt = Stmt::Function(function);
    let span = Span::union(&name, end_span);
    Ok(WithSpan::new(stmt, span))
}
*/
pub fn parse_params(it: &mut Parser) -> Result<Vec<Param>, ()> {
    let mut params: Vec<Param> = Vec::new();

    if it.check(TokenKind::RightParen) {
        // Empty parameter list
        return Ok(params);
    }

    params.push(parse_param(it)?);
    while it.check(TokenKind::Comma) {
        it.expect(TokenKind::Comma)?;
        params.push(parse_param(it)?);
    }
    Ok(params)
}

fn parse_param(it: &mut Parser) -> Result<Param, ()> {
    let name = expect_identifier(it)?;
    //it.expect(TokenKind::Colon)?;
    //let param_type = expect_identifier(it)?;
    Ok(Param { name })
}

fn parse_expr_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let expr = parse_expr(it)?;
    let end_span = it.expect(TokenKind::Semicolon)?;

    let span = Span::union(&expr, end_span);
    Ok(WithSpan::new(Stmt::Expression(Box::new(expr)), span))
}

fn parse_if_statement(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = p.expect(TokenKind::If)?;

    // Check for optional left parenthesis
    let condition = if p.check(TokenKind::LeftParen) {
        p.advance(); // Consume the left parenthesis
        let cond = parse_expr(p)?;
        p.expect(TokenKind::RightParen)?; // Ensure the closing right parenthesis
        cond
    } else {
        parse_expr(p)?
    };

    let then_branch = parse_statement(p)?;

    let mut else_branch = None;
    if p.check(TokenKind::Else) {
        p.expect(TokenKind::Else)?;
        else_branch = Some(parse_statement(p)?);
    }

    // Compute the span for the entire if statement
    let end_span = else_branch
        .as_ref()
        .map(|stmt| stmt.clone())
        .unwrap_or_else(|| then_branch.clone());

    Ok(WithSpan::new(
        Stmt::If(
            Box::new(condition),
            Box::new(then_branch),
            else_branch.map(Box::new),
        ),
        Span::union(&begin_span, &end_span),
    ))
}

pub fn parse_block_statement(p: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = p.expect(TokenKind::LeftBrace)?;
    let mut statements = vec![];
    while !p.check(TokenKind::RightBrace) && !p.check(TokenKind::Eof) {
        statements.push(parse_statement(p)?);
    }
    let end_span = p.expect(TokenKind::RightBrace)?;
    Ok(WithSpan::new(
        Stmt::Block(statements),
        Span::union(&begin_span, &end_span),
    ))
}

fn parse_thread_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let thread_span = it.expect(TokenKind::Thread)?;
    it.expect(TokenKind::LeftBrace)?;
    let mut statements = Vec::new();
    while !it.check(TokenKind::RightBrace) {
        let stmt = parse_statement(it)?;
        statements.push(stmt);
    }
    let end_span = it.expect(TokenKind::RightBrace)?;
    let stmt = Stmt::Thread(statements);
    let span = Span::union(&thread_span, end_span);
    Ok(WithSpan::new(stmt, span))
}


fn parse_let_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_span = it.expect(TokenKind::Let)?;
    let mut names = vec![expect_identifier(it)?];
    while it.check(TokenKind::Comma) {
        it.expect(TokenKind::Comma)?;
        let identifier = expect_identifier(it)?;
        names.push(identifier);
    }
    //it.expect(TokenKind::Colon)?;
    //let type_annotation = expect_identifier(it)?;
    let expr = if it.check(TokenKind::Equal) {
        it.expect(TokenKind::Equal)?;
        let expr = parse_expr(it)?;
        Some(expr)
    } else {
        None
    };
    let end_span = it.expect(TokenKind::Semicolon)?;
    let stmt = if names.len() == 1 {
        Stmt::Let(names[0].clone(), expr)
    } else {
        Stmt::LetMultiple(names)
    };
    let span = Span::union(&begin_span, &end_span);
    Ok(WithSpan::new(stmt, span))
}

fn parse_expr(it: &mut Parser) -> Result<WithSpan<Expr>, ()> {
    super::expr_parser::parse(it)
}

fn parse_print_statement(it: &mut Parser) -> Result<WithSpan<Stmt>, ()> {
    let begin_token = it.expect(TokenKind::Print)?;
    let expr = parse_expr(it)?;
    let end_token = it.expect(TokenKind::Semicolon)?;
    Ok( WithSpan::new(Stmt::Print(Box::new(expr)), Span::union(begin_token, end_token)) )
}

pub fn parse(it: &mut Parser) -> Result<Vec<WithSpan<Stmt>>, ()> {
    parse_program(it)
}

#[cfg(test)]
mod tests {
    use core::ops::Range;
    use alloc::vec;
    use alloc::string::String;
    use crate::position::Diagnostic;
    //use crate::alloc::string::ToString;

    use super::super::tokenizer::*;
    use super::*;
    fn parse_str(data: &str) -> Result<Vec<WithSpan<Stmt>>, Vec<Diagnostic>> {
        let tokens = tokenize_with_context(data);
        let mut parser = crate::parser::Parser::new(&tokens);
        match parse(&mut parser) {
            Ok(ast) => Ok(ast),
            Err(_) => Err(parser.diagnostics().to_vec()),
        }
    }

    pub fn ws<T>(value: T, range: Range<u32>) -> WithSpan<T> {
        unsafe { WithSpan::new_unchecked(value, range.start, range.end) }
    }

    fn assert_errs(data: &str, errs: &[&str]) {
        let x = parse_str(data);
        assert!(x.is_err());
        let diagnostics = x.unwrap_err();
        for diag in diagnostics {
            assert!(errs.contains(&&diag.message.as_str()), "{}", diag.message);
        }
    }

    #[test]
    fn test_url_stmt_one() {
        assert_eq!(
            parse_str("url this : \"that\";"),
            Ok(vec![
                ws(Stmt::Url(
                    Box::new(ws("this".into(), 0..4)),
                    HierarchicalName::new(vec![ws("that".into(), 4..8)]),
                ), 0..23),
            ])
        );
    }

    #[test]
    fn test_url_stmt_two() {
        assert_eq!(
            parse_str("url this : that::that2;"),
            Ok(vec![
                ws(Stmt::Url(
                    Box::new(ws("this".into(), 0..4)),
                    HierarchicalName::new(vec![ws("that".into(), 4..8), ws("that2".into(), 8..12)]),
                ), 0..23),
            ])
        );
    }

    #[test]
    fn test_expr_stmt() {
        assert_eq!(
            parse_str("nil;"),
            Ok(vec![
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))), 0..4)
            ])
        );
        assert_eq!(
            parse_str("nil;nil;"),
            Ok(vec![
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 0..3))), 0..4),
                ws(Stmt::Expression(Box::new(ws(Expr::Nil, 4..7))), 4..8),
            ])
        );
    }

    #[test]
    fn test_print_stmt() {
        assert_eq!(
            parse_str("print nil;"),
            Ok(vec![
                ws(Stmt::Print(Box::new(ws(Expr::Nil, 6..9))), 0..10),
            ])
        );
    }

    fn make_span_string(string: &str, offset: u32) -> WithSpan<String> {
        unsafe { WithSpan::new_unchecked(string.into(), offset, offset+string.len() as u32) }
    }

    #[test]
    fn test_var_decl() {
        assert_eq!(
            parse_str("let beverage;"),
            Ok(vec![
                ws(Stmt::Let(make_span_string("beverage", 4), None), 0..13),
            ])
        );
        assert_eq!(
            parse_str("let beverage = nil;"),
            Ok(vec![
                ws(Stmt::Let(
                    make_span_string("beverage", 4),
                    Some(ws(Expr::Nil, 15..18))
                ), 0..19),
            ])
        );

        unsafe {
            assert_eq!(
                parse_str("let beverage = x = nil;"),
                Ok(vec![
                    ws(Stmt::Let(
                        make_span_string("beverage", 4),
                        Some(ws(Expr::Assign(
                            WithSpan::new_unchecked("x".into(), 15, 16),
                            Box::new(ws(Expr::Nil, 19..22))
                        ), 15..22))
                    ), 0..23),
                ])
            );
        }

        assert_errs("if (nil) let beverage = nil;", &["Unexpected 'var'"]);
    }

    #[test]
    fn test_if_stmt() {
        assert_eq!(
            parse_str("if(nil) print nil;"),
            Ok(vec![
                ws(Stmt::If(
                    Box::new(ws(Expr::Nil, 3..6)),
                    Box::new(ws(Stmt::Print(Box::new(ws(Expr::Nil, 14..17))), 8..18)),
                    None,
                ), 0..18),
            ])
        );
        assert_eq!(
            parse_str("if(nil) print nil; else print false;"),
            Ok(vec![
                ws(Stmt::If(
                    Box::new(ws(Expr::Nil, 3..6)),
                    Box::new(ws(Stmt::Print(Box::new(ws(Expr::Nil, 14..17))), 8..18)),
                    Some(Box::new(
                        ws(Stmt::Print(Box::new(ws(Expr::Boolean(false), 30..35))), 24..36),
                    )),
                ), 0..36),
            ])
        );
    }

    #[test]
    fn test_block_stmt() {
        assert_eq!(parse_str("{}"), Ok(vec![
            ws(Stmt::Block(vec![]), 0..2),
        ]));
        assert_eq!(
            parse_str("{nil;}"),
            Ok(vec![
                ws(Stmt::Block(vec![
                    ws(Stmt::Expression(Box::new(
                        ws(Expr::Nil, 1..4)
                    )), 1..5),
                ]), 0..6),
            ])
        );
        assert_eq!(
            parse_str("{nil;nil;}"),
            Ok(vec![
                ws(Stmt::Block(vec![
                    ws(Stmt::Expression(Box::new(ws(Expr::Nil, 1..4))), 1..5),
                    ws(Stmt::Expression(Box::new(ws(Expr::Nil, 5..8))), 5..9),
                ]), 0..10),
            ])
        );
    }
/*
    #[test]
    fn test_use_stmt() {
        assert_eq!(parse_str("use \"mymodule\";"), Ok(vec![
            ws(Stmt::Use(
                vec![ws("mymodule".into(), 7..17)],
                vec![]
            ), 0..18),
        ]));

        assert_eq!(parse_str("import \"mymodule\" for message;"), Ok(vec![
            ws(Stmt::Use(
                vec![ws("mymodule".into(), 7..17)],
                Some(vec![
                    ws("message".into(), 22..29),
                ])
            ), 0..30),
        ]));
    }
*/
/*
    #[test]
    fn test_function_stmt() {
        unsafe {
            assert_eq!(
                parse_str("fun test(){}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![],
                        vec![]
                    ), 0..12),
                ])
            );
            assert_eq!(
                parse_str("fun test(a){}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![WithSpan::new_unchecked("a".into(), 9, 10)],
                        vec![]
                    ), 0..13),
                ])
            );
            assert_eq!(
                parse_str("fun test(){nil;}"),
                Ok(vec![
                    ws(Stmt::Function(
                        WithSpan::new_unchecked("test".into(), 4, 8),
                        vec![],
                        vec![ws(Stmt::Expression(Box::new(ws(Expr::Nil, 11..14))), 11..15),]
                    ), 0..16),
                ])
            );
        }
    }
    */
}
