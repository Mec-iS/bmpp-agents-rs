pub mod parser;
pub mod codegen;
pub mod project_builder;
pub mod validation;

pub use parser::parse_source;
pub use codegen::BmppCodeGenerator;

use anyhow::Result;

/// Convenience function to transpile BMPP source code directly into Rust code.
pub fn transpile(source: &str) -> Result<String> {
    let ast = parse_source(source)?;
    let codegen = BmppCodeGenerator::new();
    let generated_code = codegen.generate(&ast)?;
    Ok(generated_code)
}
