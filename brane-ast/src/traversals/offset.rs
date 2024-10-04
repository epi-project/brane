//  OFFSET.rs
//    by Lut99
//
//  Created:
//    12 Dec 2023, 16:33:38
//  Last edited:
//    12 Dec 2023, 17:13:38
//  Auto updated?
//    Yes
//
//  Description:
//!   Secret first first first traversal that simply updates all
//!   [`brane_dsl::TextRange`]s in the AST to have correct offsets w.r.t. the entire
//!   source (in case this is a snippet).
//

use brane_dsl::ast::{BinOp, Block, Expr, Identifier, Literal, Program, Property, PropertyExpr, Stmt, UnaOp};

use crate::errors::AstError;
use crate::state::CompileState;


/***** HELPER MACROS *****/
/// Applies an offset to the given TextRange if it is not none.
macro_rules! offset_range {
    ($range:expr, $offset:expr) => {
        if $range.is_some() {
            $range.start.line += $offset;
            $range.end.line += $offset;
        }
    };
}





/***** TRAVERSAL FUNCTIONS *****/
/// Tarverses a [`Block`] to update its range offsets.
///
/// # Arguments
/// - `block`: The [`Block`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
fn pass_block(block: &mut Block, offset: usize) {
    // Fix the range of this block itself
    offset_range!(block.range, offset);
    // ...and of all its statements, of course
    for stmt in &mut block.stmts {
        pass_stmt(stmt, offset);
    }
}

/// Tarverses a [`Stmt`] to update its range offsets.
///
/// # Arguments
/// - `stmt`: The [`Stmt`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
fn pass_stmt(stmt: &mut Stmt, offset: usize) {
    // Match the statement
    use Stmt::*;
    match stmt {
        Attribute(attr) | AttributeInner(attr) => match attr {
            brane_dsl::ast::Attribute::KeyPair { key, value, range } => {
                pass_ident(key, offset);
                pass_literal(value, offset);
                offset_range!(range, offset);
            },
            brane_dsl::ast::Attribute::List { key, values, range } => {
                pass_ident(key, offset);
                for value in values {
                    pass_literal(value, offset);
                }
                offset_range!(range, offset);
            },
        },

        Block { block } => pass_block(block, offset),

        Import { name, version, st_funcs: _, st_classes: _, attrs: _, range } => {
            pass_ident(name, offset);
            pass_literal(version, offset);
            offset_range!(range, offset);
        },
        FuncDef { ident, params, code, st_entry: _, attrs: _, range } => {
            pass_ident(ident, offset);
            for param in params {
                pass_ident(param, offset);
            }
            pass_block(code, offset);
            offset_range!(range, offset);
        },
        ClassDef { ident, props, methods, st_entry: _, symbol_table: _, attrs: _, range } => {
            pass_ident(ident, offset);
            for prop in props {
                pass_prop(prop, offset);
            }
            for method in methods {
                pass_stmt(method, offset);
            }
            offset_range!(range, offset);
        },
        Return { expr, data_type: _, output: _, attrs: _, range } => {
            if let Some(expr) = expr {
                pass_expr(expr, offset);
            }
            offset_range!(range, offset);
        },

        If { cond, consequent, alternative, attrs: _, range } => {
            pass_expr(cond, offset);
            pass_block(consequent, offset);
            if let Some(alternative) = alternative {
                pass_block(alternative, offset);
            }
            offset_range!(range, offset);
        },
        For { initializer, condition, increment, consequent, attrs: _, range } => {
            pass_stmt(initializer, offset);
            pass_expr(condition, offset);
            pass_stmt(increment, offset);
            pass_block(consequent, offset);
            offset_range!(range, offset);
        },
        While { condition, consequent, attrs: _, range } => {
            pass_expr(condition, offset);
            pass_block(consequent, offset);
            offset_range!(range, offset);
        },
        Parallel { result, blocks, merge, st_entry: _, attrs: _, range } => {
            if let Some(result) = result {
                pass_ident(result, offset);
            }
            for block in blocks {
                pass_block(block, offset);
            }
            if let Some(merge) = merge {
                pass_ident(merge, offset);
            }
            offset_range!(range, offset);
        },

        LetAssign { name, value, st_entry: _, attrs: _, range } => {
            pass_ident(name, offset);
            pass_expr(value, offset);
            offset_range!(range, offset);
        },
        Assign { name, value, st_entry: _, attrs: _, range } => {
            pass_ident(name, offset);
            pass_expr(value, offset);
            offset_range!(range, offset);
        },
        Expr { expr, data_type: _, attrs: _, range } => {
            pass_expr(expr, offset);
            offset_range!(range, offset);
        },

        Empty {} => {},
    }
}

/// Traverses an [`Identifier`] to update its range offsets.
///
/// # Arguments
/// - `ident`: The [`Identifier`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
#[inline]
fn pass_ident(ident: &mut Identifier, offset: usize) {
    let Identifier { value: _, range } = ident;
    offset_range!(range, offset);
}

/// Traverses a [`Property`] to update its range offsets.
///
/// # Arguments
/// - `prop`: The [`Property`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
fn pass_prop(prop: &mut Property, offset: usize) {
    let Property { name, data_type: _, st_entry: _, range } = prop;
    pass_ident(name, offset);
    offset_range!(range, offset);
}

/// Traverses an [`Expr`] to update its range offsets.
///
/// # Arguments
/// - `expr`: The [`Expr`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
fn pass_expr(expr: &mut Expr, offset: usize) {
    // Match the expression
    use Expr::*;
    match expr {
        Cast { expr, target: _, range } => {
            pass_expr(expr, offset);
            offset_range!(range, offset);
        },

        Call { expr, args, st_entry: _, locations: _, input: _, result: _, metadata: _, range } => {
            pass_expr(expr, offset);
            for arg in args {
                pass_expr(arg, offset);
            }
            offset_range!(range, offset);
        },
        Array { values, data_type: _, range } => {
            for value in values {
                pass_expr(value, offset);
            }
            offset_range!(range, offset);
        },
        ArrayIndex { array, index, data_type: _, range } => {
            pass_expr(array, offset);
            pass_expr(index, offset);
            offset_range!(range, offset);
        },
        Pattern { exprs, range } => {
            for expr in exprs {
                pass_expr(expr, offset);
            }
            offset_range!(range, offset);
        },

        UnaOp { op, expr, range } => {
            pass_una_op(op, offset);
            pass_expr(expr, offset);
            offset_range!(range, offset);
        },
        BinOp { op, lhs, rhs, range } => {
            pass_bin_op(op, offset);
            pass_expr(lhs, offset);
            pass_expr(rhs, offset);
            offset_range!(range, offset);
        },
        Proj { lhs, rhs, st_entry: _, range } => {
            pass_expr(lhs, offset);
            pass_expr(rhs, offset);
            offset_range!(range, offset);
        },

        Instance { name, properties, st_entry: _, range } => {
            pass_ident(name, offset);
            for property in properties {
                pass_prop_expr(property, offset);
            }
            offset_range!(range, offset);
        },
        Identifier { name, st_entry: _ } => pass_ident(name, offset),
        VarRef { name, st_entry: _ } => pass_ident(name, offset),
        Literal { literal } => pass_literal(literal, offset),

        Empty {} => {},
    }
}

/// Passes a [unary operator](UnaOp) to update its range offsets.
///
/// # Arguments
/// - `op`: The [`UnaOp`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
#[inline]
fn pass_una_op(op: &mut UnaOp, offset: usize) {
    use UnaOp::*;
    match op {
        Idx { range } | Not { range } | Neg { range } | Prio { range } => offset_range!(range, offset),
    }
}

/// Passes a [binary operator](BinOp) to update its range offsets.
///
/// # Arguments
/// - `op`: The [`BinOp`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
#[inline]
fn pass_bin_op(op: &mut BinOp, offset: usize) {
    use BinOp::*;
    match op {
        And { range }
        | Or { range }
        | Add { range }
        | Sub { range }
        | Mul { range }
        | Div { range }
        | Mod { range }
        | Eq { range }
        | Ne { range }
        | Lt { range }
        | Le { range }
        | Gt { range }
        | Ge { range } => offset_range!(range, offset),
    }
}

/// Passes a [property expression](PropertyExpr) to update its range offsets.
///
/// # Arguments
/// - `prop_expr`: The [`PropertyExpr`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
fn pass_prop_expr(prop_expr: &mut PropertyExpr, offset: usize) {
    let PropertyExpr { name, value, range } = prop_expr;
    pass_ident(name, offset);
    pass_expr(value, offset);
    offset_range!(range, offset);
}

/// Passes a [`Literal`] to update its range offsets.
///
/// # Arguments
/// - `literal`: The [`Literal`] to traverse.
/// - `offset`: The offset (in lines) to which to fix the node.
#[inline]
fn pass_literal(literal: &mut Literal, offset: usize) {
    use Literal::*;
    match literal {
        Null { range }
        | Boolean { value: _, range }
        | Integer { value: _, range }
        | Real { value: _, range }
        | String { value: _, range }
        | Semver { value: _, range }
        | Void { range } => offset_range!(range, offset),
    }
}





/***** LIBRARY *****/
/// Fixes offsets in the AST to be relative to the entire source instead of just this snippet.
///
/// # Arguments
/// - `root`: The root node of the tree on which this compiler pass will be done.
/// - `state`: The [`CompileState`] containing the offset to offset with.
///
/// # Returns
/// The same nodes as went in, but now with annotation statements translated to annotations on structs.
///
/// # Errors
/// This pass cannot error.
#[inline]
pub fn do_traversal(mut root: Program, state: &CompileState) -> Result<Program, Vec<AstError>> {
    pass_block(&mut root.block, state.offset);
    Ok(root)
}
