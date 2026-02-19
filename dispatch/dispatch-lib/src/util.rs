use ::syn::{Expr, Ident, PatPath, Path};

/// Convert an ident to an expression.
pub fn ident_to_expr(ident: Ident) -> Expr {
    Expr::Path(PatPath {
        attrs: Vec::new(),
        qself: None,
        path: Path::from(ident.clone()),
    })
}
