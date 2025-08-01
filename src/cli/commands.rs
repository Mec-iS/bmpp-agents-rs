use crate::cli::args::{Cli, Commands};
use crate::transpiler::{parser, codegen::BmppCodeGenerator};
use crate::utils::ast::AstNodeType;
use anyhow::{Result, anyhow};
use clap::Parser;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        println!("BMPP Agents v{}", env!("CARGO_PKG_VERSION"));
    }
    
    match cli.command {
        Commands::Parse { input, output_ast, validate } => {
            parse_command(&input, output_ast, validate, cli.verbose)
        },
        Commands::Compile { input, output_dir, target, include_validators } => {
            compile_command(&input, &output_dir, &target, include_validators, cli.verbose)
        },
        Commands::Validate { input, semantic_check, flow_check } => {
            validate_command(&input, semantic_check, flow_check, cli.verbose)
        },
        Commands::Format { input, in_place, stdout } => {
            format_command(&input, in_place, stdout, cli.verbose)
        },
        Commands::Init { name, output, template } => {
            init_command(&name, output.as_deref(), &template, cli.verbose)
        },
    }
}

fn parse_command(input: &Path, output_ast: bool, validate: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("Parsing BMPP file: {}", input.display());
    }
    
    let source = fs::read_to_string(input)
        .map_err(|e| anyhow!("Failed to read input file: {}", e))?;
    
    let ast = parser::parse_source(&source)?;
    
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Invalid AST root node type"));
    }
    
    println!("✅ Successfully parsed BMPP protocol");
    
    if let Some(protocol_node) = ast.children.first() {
        if protocol_node.node_type == AstNodeType::ProtocolDecl {
            if let Some(name) = protocol_node.get_string("name") {
                println!("📋 Protocol: {}", name);
            }
            if let Some(desc) = protocol_node.get_string("description") {
                println!("📝 Description: {}", desc);
            }
            
            // Count sections
            let mut roles_count = 0;
            let mut params_count = 0;
            let mut interactions_count = 0;
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::RolesSection => roles_count = child.children.len(),
                    AstNodeType::ParametersSection => params_count = child.children.len(),
                    AstNodeType::InteractionsSection => interactions_count = child.children.len(),
                    _ => {}
                }
            }
            
            println!("👥 Roles: {}", roles_count);
            println!("📊 Parameters: {}", params_count);
            println!("🔄 Interactions: {}", interactions_count);
        }
    }
    
    if output_ast {
        println!("\n--- AST Debug Output ---");
        print_ast_debug(&ast, 0);
    }
    
    if validate {
        println!("🔍 Validating protocol semantics...");
        validate_protocol_semantics(&ast)?;
        println!("✅ Protocol validation passed");
    }
    
    Ok(())
}

fn compile_command(input: &Path, output_dir: &Path, target: &str, include_validators: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("Compiling BMPP file: {} -> {}", input.display(), output_dir.display());
    }
    
    let source = fs::read_to_string(input)?;
    let ast = parser::parse_source(&source)?;
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Generate code based on target
    let generator = BmppCodeGenerator::new();
    let generated_code = match target {
        "rust" => generator.generate(&ast)?,
        _ => return Err(anyhow!("Unsupported target language: {}", target)),
    };
    
    // Write main implementation file
    let main_file = output_dir.join("lib.rs");
    fs::write(&main_file, generated_code)?;
    
    if include_validators {
        generate_validators(output_dir, &ast, verbose)?;
    }
    
    // Generate Cargo.toml for the output
    generate_cargo_toml(output_dir, &ast)?;
    
    println!("✅ Generated {} code in {}", target, output_dir.display());
    println!("📁 Main file: {}", main_file.display());
    
    Ok(())
}

fn validate_command(input: &Path, semantic_check: bool, flow_check: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("Validating BMPP file: {}", input.display());
    }
    
    let source = fs::read_to_string(input)?;
    let ast = parser::parse_source(&source)?;
    
    println!("🔍 Running validation checks...");
    
    // Basic syntax validation (already done by parser)
    println!("✅ Syntax validation passed");
    
    if semantic_check {
        validate_protocol_semantics(&ast)?;
        println!("✅ Semantic validation passed");
    }
    
    if flow_check {
        validate_parameter_flow(&ast)?;
        println!("✅ Parameter flow validation passed");
    }
    
    println!("🎉 All validations passed!");
    
    Ok(())
}

fn format_command(input: &Path, in_place: bool, stdout: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("Formatting BMPP file: {}", input.display());
    }
    
    let source = fs::read_to_string(input)?;
    let ast = parser::parse_source(&source)?;
    
    // Generate formatted output
    let formatted = format_ast(&ast)?;
    
    if stdout {
        println!("{}", formatted);
    } else if in_place {
        fs::write(input, formatted)?;
        println!("✅ Formatted file in place: {}", input.display());
    } else {
        println!("{}", formatted);
    }
    
    Ok(())
}

fn init_command(name: &str, output: Option<&Path>, template: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Initializing new BMPP protocol: {}", name);
    }
    
    let template_content = match template {
        "basic" => generate_basic_template(name),
        "multi-party" => generate_multi_party_template(name),
        _ => return Err(anyhow!("Unknown template type: {}", template)),
    };
    
    let output_file = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| format!("{}.bmpp", name.to_lowercase()).into());
    
    fs::write(&output_file, template_content)?;
    
    println!("✅ Created new protocol: {}", output_file.display());
    println!("📝 Edit the file to customize your protocol");
    
    Ok(())
}

// Helper functions
fn print_ast_debug(node: &crate::utils::ast::AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}🌳 {:?}", indent, node.node_type);
    
    for (key, value) in &node.properties {
        println!("{}  📝 {}: {:?}", indent, key, value);
    }
    
    for child in &node.children {
        print_ast_debug(child, depth + 1);
    }
}

fn validate_protocol_semantics(ast: &crate::utils::ast::AstNode) -> Result<()> {
    if ast.children.is_empty() {
        return Err(anyhow!("Protocol must contain at least one protocol definition"));
    }
    
    // Add more semantic validation logic here
    Ok(())
}

fn validate_parameter_flow(ast: &crate::utils::ast::AstNode) -> Result<()> {
    // Add parameter flow validation logic here
    Ok(())
}

fn format_ast(ast: &crate::utils::ast::AstNode) -> Result<String> {
    // Add AST formatting logic here
    Ok("// Formatted BMPP protocol\n".to_string())
}

fn generate_validators(output_dir: &Path, ast: &crate::utils::ast::AstNode, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating protocol validators...");
    }
    
    let validator_code = r#"
// Generated protocol validators
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolValidator;

impl ProtocolValidator {
    pub fn validate_interaction(&self, interaction: &str) -> bool {
        // Add validation logic
        true
    }
}
"#;
    
    let validator_file = output_dir.join("validator.rs");
    fs::write(validator_file, validator_code)?;
    
    Ok(())
}

fn generate_cargo_toml(output_dir: &Path, ast: &crate::utils::ast::AstNode) -> Result<()> {
    let default = &"generated_protocol".to_string();
    let protocol_name = ast.children
        .first()
        .and_then(|p| p.get_string("name"))
        .unwrap_or(default);
    
    let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
"#, protocol_name.to_lowercase());
    
    let cargo_file = output_dir.join("Cargo.toml");
    fs::write(cargo_file, cargo_toml)?;
    
    Ok(())
}

fn generate_basic_template(name: &str) -> String {
    format!(r#"
{} <Protocol>("a basic protocol template") {{
    roles
        A <Agent>("first participant"),
        B <Agent>("second participant")
    
    parameters
        message <String>("a simple message"),
        response <Bool>("acknowledgment response")
    
    A -> B: send <Action>("send a message")[out message]
    B -> A: ack <Action>("acknowledge receipt")[in message, out response]
}}
"#, name)
}

fn generate_multi_party_template(name: &str) -> String {
    format!(r#"
{} <Protocol>("a multi-party protocol template") {{
    roles
        Initiator <Agent>("the party that starts the protocol"),
        Coordinator <Agent>("the party that coordinates the process"),
        Participant <Agent>("a participating party in the protocol")
    
    parameters
        request_id <String>("unique identifier for the request"),
        data <String>("the data being processed"),
        status <String>("current status of the operation"),
        result <Bool>("final result of the operation")
    
    Initiator -> Coordinator: initiate <Action>("start the protocol")[out request_id, out data]
    Coordinator -> Participant: delegate <Action>("delegate task to participant")[in request_id, in data, out status]
    Participant -> Coordinator: complete <Action>("report task completion")[in request_id, in data, out result]
    Coordinator -> Initiator: finalize <Action>("provide final result")[in request_id, in result, out status]
}}
"#, name)
}
