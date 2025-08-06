pub mod codegen;
pub mod composition;
pub mod parser;
pub mod project_builder;
pub mod validation;

pub use codegen::BmppCodeGenerator;
pub use parser::parse_source;

use anyhow::Result;

/// Convenience function to transpile BMPP source code directly into Rust code.
pub fn transpile(source: &str) -> Result<String> {
    let ast = parse_source(source)?;
    let codegen = BmppCodeGenerator::new();
    let generated_code = codegen.generate(&ast)?;
    Ok(generated_code)
}
