use crate::cli::args::{Cli, Commands};
use crate::transpiler::{parser, codegen::BmppCodeGenerator};
use crate::protocol::ast::AstNodeType;
use crate::config::Config;
use crate::runtime::client::LlmClient;
use crate::runtime::llm_provider::LlmProvider;
use anyhow::{Result, anyhow};
use clap::Parser;
use std::fs;
use std::path::Path;
use crate::transpiler::validation::{validate_parameter_flow, validate_protocol_composition}; 

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        println!("BMPP Agents v{}", env!("CARGO_PKG_VERSION"));
    }
    
    match cli.command {
        Commands::Parse { input, output_ast, validate } => {
            parse_command(&input, output_ast, validate, cli.verbose)
        },
        Commands::Transpile { input, output_dir, target, include_validators } => {
            transpile_command(&input, &output_dir, &target, include_validators, cli.verbose)
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
        Commands::FromProtocol { input, output, style } => {
            from_protocol_command(&input, output.as_deref(), &style, cli.verbose)
        },
        Commands::ToProtocol { input, input_file, output, skip_validation, max_attempts } => {
            to_protocol_command(&input, input_file, output.as_deref(), skip_validation, max_attempts, cli.verbose)
        },
    }
}

fn parse_command(input: &Path, output_ast: bool, validate: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("🔍 Parsing BMPP protocol file: {}", input.display());
    }
    
    let source = fs::read_to_string(input)
        .map_err(|e| anyhow!("Failed to read input file: {}", e))?;
    
    let ast = parser::parse_source(&source)?;
    
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Invalid AST root node type"));
    }
    
    println!("✅ Successfully parsed BMPP protocol");
    
    // Extract protocol information from new AST structure
    if let Some(protocol_node) = ast.children.first() {
        if protocol_node.node_type == AstNodeType::Protocol {
            // Extract protocol name from ProtocolName child
            let mut protocol_name = "Unknown".to_string();
            let mut protocol_description = "No description".to_string();
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::ProtocolName => {
                        if let Some(name) = child.get_string("name") {
                            protocol_name = name.clone();
                        }
                    },
                    AstNodeType::Annotation => {
                        if let Some(desc) = child.get_string("description") {
                            protocol_description = desc.clone();
                        }
                    },
                    _ => {}
                }
            }
            
            println!("📋 Protocol: {}", protocol_name);
            println!("📝 Description: {}", protocol_description);
            
            // Count sections in new structure
            let mut roles_count = 0;
            let mut params_count = 0;
            let mut interactions_count = 0;
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::RolesSection => roles_count = child.children.len(),
                    AstNodeType::ParametersSection => params_count = child.children.len(),
                    AstNodeType::InteractionSection => interactions_count = child.children.len(),
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
        validate_parameter_flow(&ast)?;
        validate_protocol_composition(&ast)?;
        println!("✅ Protocol validation passed");
    }
    
    Ok(())
}

fn transpile_command(input: &Path, output_dir: &Path, target: &str, include_validators: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("🔧 Compiling BMPP file: {} -> {}", input.display(), output_dir.display());
        println!("🎯 Target: {}", target);
    }
    
    let source = fs::read_to_string(input)?;
    let ast = parser::parse_source(&source)?;
    
    // Validate before transpiling
    validate_parameter_flow(&ast)?;
    validate_protocol_composition(&ast)?;
    
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
        println!("🔍 Validating BMPP file: {}", input.display());
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
        validate_protocol_composition(&ast)?;
        println!("✅ Parameter flow validation passed");
        println!("✅ Protocol composition validation passed");
    }
    
    println!("🎉 All validations passed!");
    
    Ok(())
}

fn format_command(input: &Path, in_place: bool, stdout: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("🎨 Formatting BMPP file: {}", input.display());
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
        println!("🏗️ Initializing new BMPP protocol: {}", name);
    }
    
    let template_content = match template {
        "basic" => generate_basic_template(name),
        "multi-party" => generate_multi_party_template(name),
        "composition" => generate_composition_template(name),
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

// Ollama Integration Commands

fn from_protocol_command(
    input: &Path, 
    output: Option<&Path>, 
    style: &str, 
    verbose: bool
) -> Result<()> {
    if verbose {
        println!("🤖 Converting BMPP protocol to natural language using Ollama...");
        println!("📄 Input: {}", input.display());
        println!("🎨 Style: {}", style);
    }

    let protocol_content = fs::read_to_string(input)
        .map_err(|e| anyhow!("Failed to read protocol file: {}", e))?;

    // Validate the protocol syntax first
    let ast = parser::parse_source(&protocol_content)?;
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Invalid protocol file: not a valid BMPP protocol"));
    }

    // Initialize LLM client using existing runtime
    let config = Config::from_env();
    let llm_client = LlmClient::new(config)?;

    if verbose {
        println!("✅ Connected to Ollama at http://localhost:11434");
    }

    // Create prompt for natural language generation
    let prompt = create_from_protocol_prompt(&protocol_content, style);

    // Generate natural language description
    println!("🤖 Generating natural language description...");
    let description = llm_client.generate(&prompt)?;

    // Output the result
    if let Some(output_path) = output {
        fs::write(output_path, &description)?;
        println!("✅ Natural language description written to: {}", output_path.display());
    } else {
        println!("\n--- Generated Natural Language Description ---");
        println!("{}", description);
    }

    Ok(())
}

fn to_protocol_command(
    input: &str,
    input_file: bool,
    output: Option<&Path>,
    skip_validation: bool,
    max_attempts: u32,
    verbose: bool
) -> Result<()> {
    if verbose {
        println!("🤖 Converting natural language to BMPP protocol using Ollama...");
        println!("🔢 Max attempts: {}", max_attempts);
    }

    // Get the input description
    let description = if input_file {
        let input_path = Path::new(input);
        fs::read_to_string(input_path)
            .map_err(|e| anyhow!("Failed to read description file: {}", e))?
    } else {
        input.to_string()
    };

    if verbose {
        println!("📝 Input description length: {} characters", description.len());
    }

    // Initialize LLM client using existing runtime
    let config = Config::from_env();
    let llm_client = LlmClient::new(config)?;

    if verbose {
        println!("✅ Connected to Ollama at http://localhost:11434");
    }

    // Generate BMPP protocol with retry logic
    let mut generated_protocol = String::new();
    let mut attempt = 1;

    while attempt <= max_attempts {
        if verbose && attempt > 1 {
            println!("🔄 Attempt {} of {}...", attempt, max_attempts);
        }

        println!("🤖 Generating BMPP protocol (attempt {})...", attempt);
        
        // Create prompt for protocol generation
        let prompt = create_to_protocol_prompt(&description);
        generated_protocol = llm_client.generate(&prompt)?;

        // Validate the generated protocol if not skipped
        if !skip_validation {
            match parser::parse_source(&generated_protocol) {
                Ok(ast) => {
                    if ast.node_type == AstNodeType::Program && !ast.children.is_empty() {
                        // Run comprehensive validation
                        if let Err(e) = validate_parameter_flow(&ast) {
                            if verbose {
                                println!("⚠️  Parameter flow validation failed: {}", e);
                            }
                        } else if let Err(e) = validate_protocol_composition(&ast) {
                            if verbose {
                                println!("⚠️  Protocol composition validation failed: {}", e);
                            }
                        } else {
                            println!("✅ Generated protocol passed all validations!");
                            break;
                        }
                    } else {
                        if verbose {
                            println!("⚠️  Generated protocol structure is invalid");
                        }
                    }
                },
                Err(e) => {
                    if verbose {
                        println!("⚠️  Syntax validation failed: {}", e);
                    }
                }
            }
        } else {
            println!("⏩ Skipping validation as requested");
            break;
        }

        attempt += 1;
        if attempt > max_attempts {
            if !skip_validation {
                return Err(anyhow!("Failed to generate valid BMPP protocol after {} attempts. Try increasing --max-attempts or use --skip-validation", max_attempts));
            }
        }
    }

    // Output the result
    if let Some(output_path) = output {
        fs::write(output_path, &generated_protocol)?;
        println!("✅ BMPP protocol written to: {}", output_path.display());
    } else {
        println!("\n--- Generated BMPP Protocol ---");
        println!("{}", generated_protocol);
    }

    // Provide additional information if verbose
    if verbose {
        if let Ok(ast) = parser::parse_source(&generated_protocol) {
            display_protocol_summary(&ast);
        }
    }

    Ok(())
}

// Helper function to display protocol summary
fn display_protocol_summary(ast: &crate::protocol::ast::AstNode) {
    if let Some(protocol_node) = ast.children.first() {
        if protocol_node.node_type == AstNodeType::Protocol {
            let mut protocol_name = "Unknown".to_string();
            let mut protocol_description = "No description".to_string();
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::ProtocolName => {
                        if let Some(name) = child.get_string("name") {
                            protocol_name = name.clone();
                        }
                    },
                    AstNodeType::Annotation => {
                        if let Some(desc) = child.get_string("description") {
                            protocol_description = desc.clone();
                        }
                    },
                    _ => {}
                }
            }
            
            println!("\n📋 Protocol Name: {}", protocol_name);
            println!("📝 Description: {}", protocol_description);
            
            let mut roles_count = 0;
            let mut params_count = 0;
            let mut interactions_count = 0;
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::RolesSection => roles_count = child.children.len(),
                    AstNodeType::ParametersSection => params_count = child.children.len(),
                    AstNodeType::InteractionSection => interactions_count = child.children.len(),
                    _ => {}
                }
            }
            
            println!("👥 Roles: {}", roles_count);
            println!("📊 Parameters: {}", params_count);
            println!("🔄 Interactions: {}", interactions_count);
        }
    }
}

// Helper functions for prompt generation

fn create_from_protocol_prompt(protocol_content: &str, style: &str) -> String {
    let style_instruction = match style {
        "summary" => "Provide a brief, high-level summary of what this protocol does.",
        "detailed" => "Provide a comprehensive explanation of this protocol including its purpose, participants, data flow, and interactions.",
        "technical" => "Provide a technical analysis of this protocol including implementation details and architectural considerations.",
        _ => "Explain this protocol in clear, accessible language."
    };

    format!(r#"
You are an expert in business protocols and multi-party interactions. You are analyzing a BMPP (Blindly Meaningful Prompting Protocol) specification.

BMPP protocols follow this structure:
- Protocol declarations with semantic annotations
- Roles section defining participating agents
- Parameters section defining data types and their meanings
- Interactions section defining message flows between agents
- Protocol composition for calling other protocols

Here is the BMPP protocol to analyze:

{}

Task: {}

Please provide a clear, well-structured explanation that covers:
1. The overall purpose and context of the protocol
2. The roles and responsibilities of each participant
3. The data being exchanged and its significance
4. The step-by-step flow of interactions
5. Any protocol compositions and their purpose
6. Any important constraints or business rules

Focus on making this understandable to both technical and business stakeholders.
"#, protocol_content.trim(), style_instruction)
}

fn create_to_protocol_prompt(description: &str) -> String {
    format!(r#"
You are an expert protocol designer specializing in BMPP (Business Multi-Party Protocol) specifications. Your task is to convert natural language descriptions into formal BMPP protocol syntax.

BMPP Protocol Syntax:

ProtocolName <Protocol>("description of the protocol") {{
    roles
        RoleName <Agent>("description of this role"),
        AnotherRole <Agent>("description of this role")

    parameters
        param_name <Type>("semantic meaning of this parameter"),
        another_param <Type>("semantic meaning of this parameter")

    RoleA -> RoleB: action_name <Action>("description of this interaction")[in param1, out param2]
    RoleB -> RoleA: response_action <Action>("description of this interaction")[in param2, out param3]
    SubProtocol <Enactment>(RoleA, RoleB, in param1, out result)
}}

Key rules:
- All descriptions must be enclosed in parentheses: ("description")
- Available types: String, Int, Float, Bool
- Parameter flows use 'in' for inputs and 'out' for outputs
- Each interaction must specify parameter directions
- Role names, parameter names, and action names should be descriptive identifiers
- Protocol composition uses <Enactment> syntax for calling other protocols
- Ensure all referenced parameters are declared in the parameters section

Natural language description to convert:

{}

Generate a complete, valid BMPP protocol that captures all the essential elements described above. Make sure to:
1. Choose appropriate role names that reflect the participants
2. Define all necessary parameters with clear semantic meanings
3. Model the interaction flow accurately
4. Use meaningful action names that describe what happens
5. Ensure parameter flows are logically consistent
6. Use protocol composition where appropriate for complex workflows

Respond with ONLY the BMPP protocol syntax, no additional explanation.
"#, description.trim())
}

// Helper functions

fn print_ast_debug(node: &crate::protocol::ast::AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}🌳 {:?}", indent, node.node_type);
    
    for (key, value) in &node.properties {
        println!("{}  📝 {}: {:?}", indent, key, value);
    }
    
    for child in &node.children {
        print_ast_debug(child, depth + 1);
    }
}

fn validate_protocol_semantics(ast: &crate::protocol::ast::AstNode) -> Result<()> {
    if ast.children.is_empty() {
        return Err(anyhow!("Protocol must contain at least one protocol definition"));
    }
    
    for protocol_node in &ast.children {
        if protocol_node.node_type == AstNodeType::Protocol {
            // Validate that protocol has required sections
            let mut has_roles = false;
            let mut has_parameters = false;
            let mut has_interactions = false;
            
            for child in &protocol_node.children {
                match child.node_type {
                    AstNodeType::RolesSection => has_roles = true,
                    AstNodeType::ParametersSection => has_parameters = true,
                    AstNodeType::InteractionSection => has_interactions = true,
                    _ => {}
                }
            }
            
            if !has_roles {
                return Err(anyhow!("Protocol must have a roles section"));
            }
            if !has_parameters {
                return Err(anyhow!("Protocol must have a parameters section"));
            }
            if !has_interactions {
                return Err(anyhow!("Protocol must have interactions"));
            }
        }
    }
    
    Ok(())
}

fn format_ast(_ast: &crate::protocol::ast::AstNode) -> Result<String> {
    // TODO: Implement proper formatting based on new AST structure
    Ok("// Formatted BMPP protocol\n// (Formatting not yet implemented)\n".to_string())
}

fn generate_validators(output_dir: &Path, _ast: &crate::protocol::ast::AstNode, verbose: bool) -> Result<()> {
    if verbose {
        println!("🔧 Generating protocol validators...");
    }
    
    let validator_code = r#"
// Generated protocol validators
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolValidator;

impl ProtocolValidator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_interaction(&self, interaction: &str) -> Result<bool> {
        // Add validation logic based on BSPL rules
        Ok(true)
    }
    
    pub fn validate_parameter_flow(&self, from_role: &str, to_role: &str, parameters: &[String]) -> Result<bool> {
        // Add parameter flow validation
        Ok(true)
    }
    
    pub fn validate_protocol_composition(&self, protocol_name: &str, roles: &[String]) -> Result<bool> {
        // Add composition validation
        Ok(true)
    }
}
"#;
    
    let validator_file = output_dir.join("validator.rs");
    fs::write(validator_file, validator_code)?;
    
    Ok(())
}

fn generate_cargo_toml(output_dir: &Path, ast: &crate::protocol::ast::AstNode) -> Result<()> {
    let default = &"generated_protocol".to_string();
    let mut protocol_name = default.clone();
    
    // Extract protocol name from new AST structure
    if let Some(protocol_node) = ast.children.first() {
        if protocol_node.node_type == AstNodeType::Protocol {
            for child in &protocol_node.children {
                if child.node_type == AstNodeType::ProtocolName {
                    if let Some(name) = child.get_string("name") {
                        protocol_name = name.clone();
                        break;
                    }
                }
            }
        }
    }
    
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
"#, protocol_name.to_lowercase());
    
    let cargo_file = output_dir.join("Cargo.toml");
    fs::write(cargo_file, cargo_toml)?;
    
    Ok(())
}

fn generate_basic_template(name: &str) -> String {
    format!(r#"{} <Protocol>("a basic protocol template") {{
    roles
        A <Agent>("first participant"),
        B <Agent>("second participant")
    
    parameters
        message <String>("a simple message"),
        response <Bool>("acknowledgment response")
    
    A -> B: send <Action>("send a message")[out message]
    B -> A: ack <Action>("acknowledge receipt")[in message, out response]
}}"#, name)
}

fn generate_multi_party_template(name: &str) -> String {
    format!(r#"{} <Protocol>("a multi-party protocol template") {{
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
}}"#, name)
}

fn generate_composition_template(name: &str) -> String {
    format!(r#"{} <Protocol>("a protocol template with composition") {{
    roles
        Client <Agent>("the requesting party"),
        Server <Agent>("the service provider"),
        Processor <Agent>("the data processor")
    
    parameters
        request_id <String>("unique identifier for the request"),
        data <String>("input data for processing"),
        processed_data <String>("the processed result"),
        confirmation <Bool>("processing confirmation")
    
    Client -> Server: request <Action>("initiate processing request")[out request_id, out data]
    ProcessingSubProtocol <Enactment>[Server, Processor, in request_id, in data, out processed_data]
    Server -> Client: respond <Action>("return processed result")[in request_id, in processed_data, out confirmation]
}}

ProcessingSubProtocol <Protocol>("handles the actual data processing") {{
    roles
        Coordinator <Agent>("coordinates the processing"),
        Worker <Agent>("performs the processing work")
    
    parameters
        task_id <String>("processing task identifier"),
        input <String>("data to be processed"),
        output <String>("processed output")
    
    Coordinator -> Worker: process <Action>("request data processing")[in task_id, in input]
    Worker -> Coordinator: complete <Action>("return processed data")[in task_id, out output]
}}"#, name)
}

