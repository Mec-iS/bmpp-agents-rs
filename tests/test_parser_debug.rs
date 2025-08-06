use anyhow::Result;
use bmpp_agents::transpiler::parser::parse_source;

#[test]
fn test_debug_invalid_direction() {
    let bmpp_source = r#"
InvalidDirectionProtocol <Protocol>("protocol with invalid direction") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        param1 <String>("test parameter")
    
    A -> B: action1 <Action>("action with invalid direction")[invalid param1]
}
        "#;

    let result = parse_source(bmpp_source);

    match result {
        Ok(ast) => {
            println!("❌ Parser unexpectedly succeeded!");
            debug_ast(&ast, 0);
        }
        Err(e) => {
            println!("✅ Parser correctly failed with error: {}", e);
        }
    }
}

fn debug_ast(node: &bmpp_agents::protocol::ast::AstNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}AstNode::{:?}", indent, node.node_type);

    // Print properties if any
    for (key, value) in &node.properties {
        println!("{}  {}: {:?}", indent, key, value);
    }

    // Recursively print children
    for child in &node.children {
        debug_ast(child, depth + 1);
    }
}

#[test]
fn test_valid_direction_for_comparison() -> Result<()> {
    let bmpp_source = r#"
ValidDirectionProtocol <Protocol>("protocol with valid direction") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        param1 <String>("test parameter")
    
    A -> B: action1 <Action>("action with valid direction")[out param1]
}
        "#;

    let result = parse_source(bmpp_source);
    assert!(
        result.is_ok(),
        "Valid direction should parse successfully: {:?}",
        result
    );

    // Additional validation that the AST is properly structured
    if let Ok(ast) = result {
        assert_eq!(
            ast.node_type,
            bmpp_agents::protocol::ast::AstNodeType::Program
        );
        assert_eq!(ast.children.len(), 1, "Should have exactly one protocol");

        let protocol = &ast.children[0];
        assert_eq!(
            protocol.node_type,
            bmpp_agents::protocol::ast::AstNodeType::Protocol
        );

        println!("✅ Valid protocol parsed successfully with proper AST structure");
    }

    Ok(())
}
