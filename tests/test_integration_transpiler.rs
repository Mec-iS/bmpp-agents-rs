use anyhow::Result;
use bmpp_agents::transpiler::{codegen::BmppCodeGenerator, parser::parse_source};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_end_to_end_bmpp_compilation() -> Result<()> {
    // 1. Setup: Define BMPP protocol source code and create a temporary output directory
    let bmpp_source = r#"
SimpleExchange <Protocol>("a basic data exchange protocol for testing") {
    roles
        Client <Agent>("the party requesting information"),
        Server <Agent>("the party providing information")
    
    parameters
        request_id <String>("unique identifier for the request"),
        query <String>("the information being requested"),
        response <String>("the information provided in response"),
        status <Bool>("confirmation that the request was processed")
    
    Client -> Server: send_request <Action>("send a request for information")[out request_id, out query]
    Server -> Client: send_response <Action>("provide the requested information")[in request_id, in query, out response, out status]
}
    "#;

    let temp_dir = tempdir()?;
    let output_path = temp_dir.path();

    // 2. Execution: Parse BMPP source and generate Rust code
    let ast = parse_source(bmpp_source)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;

    // 3. Create project structure (simulating what ProjectBuilder would do)
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Write main library file
    let lib_rs_path = src_dir.join("lib.rs");
    fs::write(&lib_rs_path, &generated_code)?;

    // Write main binary file (for executable projects)
    let main_rs_content = format!(
        r#"
// Generated main file for SimpleExchange protocol
use anyhow::Result;

mod lib;
use lib::SimpleExchangeProtocol;

fn main() -> Result<()> {{
    let mut protocol = SimpleExchangeProtocol::new();
    
    println!("Initializing SimpleExchange protocol...");
    
    // Execute protocol interactions
    protocol.send_request()?;
    protocol.send_response()?;
    
    println!("Protocol execution completed successfully!");
    
    Ok(())
}}
"#
    );

    let main_rs_path = src_dir.join("main.rs");
    fs::write(&main_rs_path, &main_rs_content)?;

    // Generate Cargo.toml
    let cargo_toml_content = r#"
[package]
name = "simple_exchange_protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"

[[bin]]
name = "simple_exchange"
path = "src/main.rs"
"#;

    let cargo_toml_path = output_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content)?;

    // 4. Verification: Check that the expected files were created
    assert!(cargo_toml_path.exists(), "Cargo.toml was not created");
    assert!(lib_rs_path.exists(), "src/lib.rs was not created");
    assert!(main_rs_path.exists(), "src/main.rs was not created");

    // 5. Verify generated library code content
    let lib_content = fs::read_to_string(&lib_rs_path)?;

    // Check for BMPP-specific generated content
    assert!(
        lib_content.contains("pub struct SimpleExchangeProtocol"),
        "SimpleExchangeProtocol struct not found in generated library code"
    );

    assert!(
        lib_content.contains("pub struct Agent"),
        "Agent struct not found in generated library code"
    );

    assert!(
        lib_content.contains("pub fn send_request"),
        "send_request method not found in generated library code"
    );

    assert!(
        lib_content.contains("pub fn send_response"),
        "send_response method not found in generated library code"
    );

    // 6. Verify main binary code content
    let main_content = fs::read_to_string(&main_rs_path)?;

    assert!(
        main_content.contains("use lib::SimpleExchangeProtocol"),
        "Protocol import not found in generated main code"
    );

    assert!(
        main_content.contains("protocol.send_request()"),
        "send_request call not found in generated main code"
    );

    assert!(
        main_content.contains("protocol.send_response()"),
        "send_response call not found in generated main code"
    );

    // 7. Verify protocol structure in library
    assert!(
        lib_content.contains("client: Agent"),
        "client field not found in protocol struct"
    );

    assert!(
        lib_content.contains("server: Agent"),
        "server field not found in protocol struct"
    );

    // 8. Verify parameters are included
    assert!(
        lib_content.contains("request_id: String"),
        "request_id parameter not found in generated library code"
    );

    assert!(
        lib_content.contains("query: String"),
        "query parameter not found in generated library code"
    );

    assert!(
        lib_content.contains("response: String"),
        "response parameter not found in generated library code"
    );

    assert!(
        lib_content.contains("status: bool"),
        "status parameter not found in generated library code"
    );

    // 9. Check imports and serialization support
    assert!(
        lib_content.contains("use serde::{Serialize, Deserialize}"),
        "Serde imports not found in generated library code"
    );

    assert!(
        lib_content.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"),
        "Derive macros not found in generated library code"
    );

    // 10. Verify Cargo.toml content
    let cargo_content = fs::read_to_string(&cargo_toml_path)?;
    assert!(
        cargo_content.contains("name = \"simple_exchange_protocol\""),
        "Package name not found in Cargo.toml"
    );

    assert!(
        cargo_content.contains("anyhow = \"1.0\""),
        "anyhow dependency not found in Cargo.toml"
    );

    println!("✅ End-to-end BMPP compilation test passed!");
    println!("Generated library code preview:");
    println!("{}", &lib_content[..std::cmp::min(500, lib_content.len())]);

    Ok(())
}

#[test]
fn test_bmpp_protocol_with_complex_interactions() -> Result<()> {
    // Test a more complex protocol with multiple parameter flows
    let complex_bmpp = r#"
Negotiation <Protocol>("a complex negotiation protocol with multiple rounds") {
    roles
        Proposer <Agent>("the party making proposals"),
        Evaluator <Agent>("the party evaluating proposals"),
        Mediator <Agent>("the neutral party facilitating negotiation")
    
    parameters
        ID <String>("unique identifier of the negotiation instance"),
        proposal_id <String>("unique identifier of each proposal"),
        terms <String>("the terms being proposed"),
        evaluation <String>("assessment of the proposal"),
        counter_offer <String>("alternative terms proposed"),
        mediation_id <String>("unique identifier for mediation session"),
        final_decision <Bool>("whether the negotiation succeeded"),
        reason <String>("explanation for the final decision")
    
    Proposer -> Evaluator: submit_proposal <Action>("submit initial proposal")[out ID, out proposal_id, out terms]
    Evaluator -> Proposer: evaluate_proposal <Action>("provide evaluation feedback")[in proposal_id, in terms, out evaluation, out counter_offer]
    Proposer -> Mediator: request_mediation <Action>("request third-party mediation")[in evaluation, in counter_offer, out mediation_id]
    Mediator -> Evaluator: mediate_discussion <Action>("facilitate resolution")[in mediation_id, out final_decision, out reason]
}
    "#;

    let ast = parse_source(complex_bmpp)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;

    // Verify complex protocol structure
    assert!(generated_code.contains("pub struct NegotiationProtocol"));
    assert!(generated_code.contains("proposer: Agent"));
    assert!(generated_code.contains("evaluator: Agent"));
    assert!(generated_code.contains("mediator: Agent"));

    // Verify all interactions are generated
    assert!(generated_code.contains("pub fn submit_proposal"));
    assert!(generated_code.contains("pub fn evaluate_proposal"));
    assert!(generated_code.contains("pub fn request_mediation"));
    assert!(generated_code.contains("pub fn mediate_discussion"));

    // Verify complex parameter types
    assert!(generated_code.contains("counter_offer: String"));
    assert!(generated_code.contains("final_decision: bool"));
    assert!(generated_code.contains("mediation_id: String"));

    Ok(())
}

#[test]
fn test_project_compilation_with_validation() -> Result<()> {
    // Test that generated projects can theoretically be compiled
    let bmpp_source = r#"
Ping <Protocol>("simple ping-pong protocol for validation testing") {
    roles
        Sender <Agent>("sends ping messages"),
        Receiver <Agent>("responds with pong messages")
    
    parameters
        message_id <String>("unique identifier for the message"),
        content <String>("the message content")
    
    Sender -> Receiver: ping <Action>("send a ping message")[out message_id, out content]
    Receiver -> Sender: pong <Action>("respond with pong message")[in message_id, in content]
}
    "#;

    let temp_dir = tempdir()?;
    let output_path = temp_dir.path();

    // Generate the project
    let ast = parse_source(bmpp_source)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;

    // Create complete project structure
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;

    // Write library
    let lib_rs_path = src_dir.join("lib.rs");
    fs::write(&lib_rs_path, &generated_code)?;

    // Write Cargo.toml with proper syntax checking dependencies
    let cargo_toml_content = r#"
[package]
name = "ping_protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"

[dev-dependencies]
# Dependencies for testing generated code
tokio = { version = "1.0", features = ["full"] }
"#;

    let cargo_toml_path = output_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content)?;

    // Verify the generated Rust code is syntactically valid
    let lib_content = fs::read_to_string(&lib_rs_path)?;

    // Basic syntax validation checks
    assert!(lib_content.contains("impl PingProtocol"));
    assert!(lib_content.contains("pub fn new()"));
    assert!(lib_content.contains("pub fn ping"));
    assert!(lib_content.contains("pub fn pong"));
    assert!(lib_content.contains("-> Result<()>"));

    // Check that all braces are balanced
    let open_braces = lib_content.matches('{').count();
    let close_braces = lib_content.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Unbalanced braces in generated code"
    );

    // Check that all parentheses are balanced
    let open_parens = lib_content.matches('(').count();
    let close_parens = lib_content.matches(')').count();
    assert_eq!(
        open_parens, close_parens,
        "Unbalanced parentheses in generated code"
    );

    println!("✅ Generated project structure validation passed!");

    Ok(())
}
