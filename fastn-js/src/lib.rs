extern crate self as fastn_js;

mod func;
mod instruction;
mod mutable_variable;
mod ssr;
mod static_variable;

pub use func::{func0, func1, func2, Func};
pub use instruction::Statement;
pub use mutable_variable::{mutable_quoted, mutable_unquoted, MutableVariable};
pub use ssr::{ssr, ssr_str};
pub use static_variable::{static_quoted, static_unquoted, StaticVariable};

pub fn encode(js: &[fastn_js::Func]) -> String {
    let mut w = Vec::new();
    let o = pretty::RcDoc::intersperse(js.iter().map(|f| f.to_js()), pretty::RcDoc::space());
    o.render(80, &mut w).unwrap();
    String::from_utf8(w).unwrap()
}
