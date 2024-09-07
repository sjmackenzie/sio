use werbolg_core::{Statement, Module};
use crate::{
    Ast,
    ast::Stmt,
};
use alloc::{
    boxed::Box,
    vec::Vec,
};
/*
pub fn convert_ast_to_module(ast: Ast) -> Module {
    let mut statements = Vec::new();
    
    for with_span_stmt in ast {
        // Extract the inner Stmt
        let stmt = with_span_stmt.value;
        // Convert Stmt to Statement and append to main statements vector
        convert_and_append_stmt(&stmt, &mut statements);
    }
    
    // Create the Module struct with the collected statements
    Module {
        statements,
    }
}

fn convert_and_append_stmt(stmt: &Stmt, statements: &mut Vec<Statement>) {
    match stmt {
        Stmt::Url(identifier, hierarchical_name) => {
            statements.push(Statement::Url((*identifier.value).clone(), hierarchical_name.clone()));
        }
        Stmt::Expression(expr) => {
            statements.push(Statement::Expression((*expr.value).clone()));
        }
        Stmt::Print(expr) => {
            statements.push(Statement::Print((*expr.value).clone()));
        }
        Stmt::If(cond, then_block, else_block) => {
            statements.push(Statement::If(
                (*cond.value).clone(),
                Box::new(convert_stmt_to_statement(&then_block.value)),
                else_block.as_ref().map(|else_block| Box::new(convert_stmt_to_statement(&else_block.value))),
            ));
        }
        Stmt::Block(stmts) => {
            for stmt in stmts {
                convert_and_append_stmt(&stmt.value, statements);
            }
        }
        Stmt::Let(identifier, expr) => {
            statements.push(Statement::Let(identifier.value.clone(), expr.as_ref().map(|expr| (*expr.value).clone())));
        }
        Stmt::Thread(stmts) => {
            for stmt in stmts {
                convert_and_append_stmt(&stmt.value, statements);
            }
        }
        Stmt::Function(function) => {
            // Assume function can be differentiated as either definition or implementation
            if function.is_definition() {
                statements.push(Statement::FunDef(function.clone()));
            } else {
                statements.push(Statement::FunImpl(function.clone()));
            }
        }
        Stmt::Use(module_name, identifiers) => {
            statements.push(Statement::Use(module_name.value.clone(), identifiers.clone().map(|ids| ids.iter().map(|id| id.value.clone()).collect())));
        }
        Stmt::CorporalModule(module) => {
            for stmt in &module.statements {
                convert_and_append_stmt(&stmt.value, statements);
            }
        }
        Stmt::MajorModule(module) => {
            for stmt in &module.statements {
                convert_and_append_stmt(&stmt.value, statements);
            }
        }
        Stmt::BrigadierModule(module) => {
            for stmt in &module.statements {
                convert_and_append_stmt(&stmt.value, statements);
            }
        }
    }
}

fn convert_stmt_to_statement(stmt: &Stmt) -> Statement {
    match stmt {
        Stmt::Url(identifier, hierarchical_name) => {
            Statement::Url((*identifier.value).clone(), hierarchical_name.clone())
        }
        Stmt::Expression(expr) => {
            Statement::Expression((*expr.value).clone())
        }
        Stmt::Print(expr) => {
            Statement::Print((*expr.value).clone())
        }
        Stmt::If(cond, then_block, else_block) => {
            Statement::If(
                (*cond.value).clone(),
                Box::new(convert_stmt_to_statement(&then_block.value)),
                else_block.as_ref().map(|else_block| Box::new(convert_stmt_to_statement(&else_block.value))),
            )
        }
        Stmt::Let(identifier, expr) => {
            Statement::Let(identifier.value.clone(), expr.as_ref().map(|expr| (*expr.value).clone()))
        }
        Stmt::Function(function) => {
            if function.is_definition() {
                Statement::FunDef(function.clone())
            } else {
                Statement::FunImpl(function.clone())
            }
        }
        Stmt::Use(module_name, identifiers) => {
            Statement::Use(module_name.value.clone(), identifiers.clone().map(|ids| ids.iter().map(|id| id.value.clone()).collect()))
        }
    }
}


pub fn module(fileunit: &FileUnit) -> Result<ir::Module, Vec<ParseError>> {
    let m = parse::module(&fileunit.content).map_err(|errs| {
        errs.into_iter()
            .map(|err| simple_to_perr(err))
            .collect::<Vec<_>>()
    })?;

    let statements = m
        .into_iter()
        .map(|(n, span, fun)| {
            let body = rewrite_expr(&fun.body);
            Statement::Function(
                span,
                ir::FunDef {
                    privacy: ir::Privacy::Public,
                    name: ir::Ident::from(n),
                },
                ir::FunImpl {
                    vars: fun.args,
                    body,
                },
            )
        })
        .collect::<Vec<_>>();

    Ok(ir::Module { statements })
}

fn rewrite_expr_spanbox(span_expr: &(Stmt, parse::Span)) -> Box<Spanned<ir::Expr>> {
    let span = span_expr.1.clone();
    let expr = rewrite_expr(span_expr);
    Box::new(Spanned::new(span, expr))
}

fn rewrite_expr(span_expr: &(Stmt, parse::Span)) -> ir::Expr {
    match &span_expr.0 {
        Stmt::Error => todo!(),
        Stmt::Literal(lit) => ir::Expr::Literal(span_expr.1.clone(), lit.clone()),
        Stmt::List(list) => ir::Expr::Sequence(
            span_expr.1.clone(),
            list.iter().map(|se| rewrite_expr(se)).collect::<Vec<_>>(),
        ),
        Stmt::Local(l) => ir::Expr::Path(
            span_expr.1.clone(),
            ir::Path::relative(ir::Ident::from(l.as_str())),
        ),
        Stmt::Let(name, bind, then) => ir::Expr::Let(
            ir::Binder::Ident(ir::Ident::from(name.as_str())),
            Box::new(rewrite_expr(bind)),
            Box::new(rewrite_expr(then)),
        ),
        Stmt::Then(first, second) => ir::Expr::Let(
            ir::Binder::Ignore,
            Box::new(rewrite_expr(first)),
            Box::new(rewrite_expr(second)),
        ),
        Stmt::Binary(left, op, right) => ir::Expr::Call(
            span_expr.1.clone(),
            vec![
                ir::Expr::Path(
                    /* should be op's span */ span_expr.1.clone(),
                    ir::Path::absolute(ir::Ident::from(op.as_str())),
                ),
                rewrite_expr(&left),
                rewrite_expr(&right),
            ],
        ),
        Stmt::Call(x, args) => {
            let mut exprs = vec![rewrite_expr(x)];
            for a in args {
                exprs.push(rewrite_expr(a))
            }
            ir::Expr::Call(span_expr.1.clone(), exprs)
        }
        Stmt::If(cond, then_expr, else_expr) => ir::Expr::If {
            span: span_expr.1.clone(),
            cond: rewrite_expr_spanbox(cond),
            then_expr: rewrite_expr_spanbox(then_expr),
            else_expr: rewrite_expr_spanbox(else_expr),
        },
    }
}*/