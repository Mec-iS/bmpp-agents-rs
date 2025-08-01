use anyhow::Result;
use tempfile::tempdir;
use bmpp_agents::transpiler::{parser::parse_source, codegen::BmppCodeGenerator};
use std::fs;

#[test]
fn test_end_to_end_bmpp_compilation() -> Result<()> {
    // 1. Setup: Define BMPP protocol source code and create a temporary output directory
    let bmpp_source = r#"
Purchase <Protocol>("a basic purchase protocol for testing") {
    roles
        Buyer <Agent>("the party wanting to buy an item"),
        Seller <Agent>("the party selling the item")
    
    parameters
        item_id <String>("unique identifier for the item"),
        item_name <String>("the name or description of the product"),
        price <Float>("the cost of the item quoted by the seller"),
        accept <Bool>("confirmation that the buyer agrees to the quote")
    
    Buyer -> Seller: request_quote <Action>("request for a price quote")[out item_id, out item_name]
    Seller -> Buyer: provide_quote <Action>("provide a price quote for requested item")[in item_id, in item_name, out price]
    Buyer -> Seller: accept_quote <Action>("accept the seller's price quote")[in item_id, in price, out accept]
}
    "#;
    
    let temp_dir = tempdir()?;
    let output_path = temp_dir.path();

    // 2. Execution: Parse BMPP source and generate Rust code
    let ast = parse_source(bmpp_source)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;

    // 3. Create project structure manually (since we don't have ProjectBuilder in current implementation)
    let src_dir = output_path.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Write generated library code
    let lib_rs_path = src_dir.join("lib.rs");
    fs::write(&lib_rs_path, &generated_code)?;
    
    // Generate Cargo.toml
    let cargo_toml_content = r#"
[package]
name = "generated_purchase_protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
"#;
    
    let cargo_toml_path = output_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, cargo_toml_content)?;

    // 4. Verification: Check that the expected files were created
    assert!(cargo_toml_path.exists(), "Cargo.toml was not created");
    assert!(lib_rs_path.exists(), "src/lib.rs was not created");

    // 5. Verify generated code content
    let generated_content = fs::read_to_string(&lib_rs_path)?;
    
    // Check for BMPP-specific generated content
    assert!(
        generated_content.contains("pub struct PurchaseProtocol"),
        "PurchaseProtocol struct not found in generated code"
    );
    
    assert!(
        generated_content.contains("pub struct Agent"),
        "Agent struct not found in generated code"
    );
    
    assert!(
        generated_content.contains("pub fn request_quote"),
        "request_quote method not found in generated code"
    );
    
    assert!(
        generated_content.contains("pub fn provide_quote"),
        "provide_quote method not found in generated code"
    );
    
    assert!(
        generated_content.contains("pub fn accept_quote"),
        "accept_quote method not found in generated code"
    );

    // 6. Verify protocol structure
    assert!(
        generated_content.contains("buyer: Agent"),
        "buyer field not found in protocol struct"
    );
    
    assert!(
        generated_content.contains("seller: Agent"),
        "seller field not found in protocol struct"
    );
    
    // 7. Verify parameters are included
    assert!(
        generated_content.contains("item_id: String"),
        "item_id parameter not found in generated code"
    );
    
    assert!(
        generated_content.contains("price: f64"),
        "price parameter not found in generated code"
    );

    // 8. Check imports and serialization support
    assert!(
        generated_content.contains("use serde::{Serialize, Deserialize}"),
        "Serde imports not found in generated code"
    );
    
    assert!(
        generated_content.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"),
        "Derive macros not found in generated code"
    );

    println!("✅ End-to-end BMPP compilation test passed!");
    println!("Generated code preview:");
    println!("{}", &generated_content[..std::cmp::min(500, generated_content.len())]);

    Ok(())
}

#[test]
fn test_parser_with_invalid_bmpp_syntax() {
    // Test that parser properly rejects invalid BMPP syntax
    let invalid_bmpp = r#"
InvalidProtocol <Protocol> "missing parentheses around annotation" {
    roles
        A <Agent>("valid role")
    parameters
        id <String>("valid parameter")
    A -> A: test <Action>("valid action")[out id]
}
    "#;
    
    let result = parse_source(invalid_bmpp);
    assert!(result.is_err(), "Parser should reject invalid syntax");
}

#[test]
fn test_parser_with_missing_mandatory_sections() {
    // Test that parser requires all mandatory sections
    let incomplete_bmpp = r#"
Test <Protocol>("test protocol missing parameters section") {
    roles
        A <Agent>("test agent")
    
    A -> A: test <Action>("test action")[]
}
    "#;
    
    let result = parse_source(incomplete_bmpp);
    assert!(result.is_err(), "Parser should require all mandatory sections");
}

#[test]
fn test_multi_party_protocol_generation() -> Result<()> {
    let multi_party_bmpp = r#"
Auction <Protocol>("a three-party auction protocol") {
    roles
        Bidder <Agent>("the party placing bids"),
        Auctioneer <Agent>("the party managing the auction"),
        Winner <Agent>("the party who wins the auction")
    
    parameters
        item_id <String>("unique identifier for the auction item"),
        bid_amount <Float>("the amount being bid"),
        winner_id <String>("identifier of the winning bidder"),
        final_price <Float>("the final winning bid amount")
    
    Bidder -> Auctioneer: place_bid <Action>("place a bid on an item")[out item_id, out bid_amount]
    Auctioneer -> Winner: declare_winner <Action>("declare the auction winner")[in item_id, out winner_id, out final_price]
    Winner -> Auctioneer: confirm_win <Action>("confirm acceptance of winning bid")[in winner_id, in final_price]
}
    "#;
    
    let ast = parse_source(multi_party_bmpp)?;
    let code_generator = BmppCodeGenerator::new();
    let generated_code = code_generator.generate(&ast)?;
    
    // Verify multi-party structure
    assert!(generated_code.contains("pub struct AuctionProtocol"));
    assert!(generated_code.contains("bidder: Agent"));
    assert!(generated_code.contains("auctioneer: Agent"));
    assert!(generated_code.contains("winner: Agent"));
    
    // Verify all interactions are generated
    assert!(generated_code.contains("pub fn place_bid"));
    assert!(generated_code.contains("pub fn declare_winner"));
    assert!(generated_code.contains("pub fn confirm_win"));
    
    Ok(())
}
