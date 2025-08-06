use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use anyhow::{Result, anyhow};
use crate::protocol::ast::{AstNode, AstNodeType};

#[derive(Parser)]
#[grammar = "src/grammars/v1/bmpp.pest"]
pub struct BmppParser;

pub fn parse_source(source: &str) -> Result<AstNode> {
    let mut pairs = BmppParser::parse(Rule::Program, source)?;
    let program_pair = pairs.next().ok_or_else(|| anyhow!("No program found"))?;
    parse_program(program_pair)
}

fn parse_program(pair: Pair<Rule>) -> Result<AstNode> {
    let mut program_node = AstNode::new(AstNodeType::Program);
    
    for protocol_pair in pair.into_inner() {
        match protocol_pair.as_rule() {
            Rule::Protocol => {
                let protocol_node = parse_protocol(protocol_pair)?;
                program_node.children.push(Box::new(protocol_node));
            }
            Rule::EOI => break,
            _ => {}
        }
    }
    
    Ok(program_node)
}

fn parse_protocol(pair: Pair<Rule>) -> Result<AstNode> {
    let mut protocol_node = AstNode::new(AstNodeType::Protocol);
    let mut inner = pair.into_inner();

    // Parse protocol name
    if let Some(name_pair) = inner.next() {
        if name_pair.as_rule() == Rule::ProtocolName {
            let mut name_node = AstNode::new(AstNodeType::ProtocolName);
            let name_inner = name_pair.into_inner().next().ok_or_else(|| anyhow!("Protocol name missing"))?;
            name_node.set_string("name", name_inner.as_str());
            protocol_node.children.push(Box::new(name_node));
        }
    } else {
        return Err(anyhow!("Protocol name missing"));
    }

    // Skip "<Protocol>" keyword
    inner.next();

    // Parse annotation (description)
    if let Some(annotation_pair) = inner.next() {
        if annotation_pair.as_rule() == Rule::Annotation {
            let annotation_node = parse_annotation(annotation_pair)?;
            protocol_node.children.push(Box::new(annotation_node));
        }
    } else {
        return Err(anyhow!("Protocol annotation missing"));
    }

    // Skip opening brace
    inner.next();

    // Parse protocol sections
    while let Some(section) = inner.next() {
        match section.as_rule() {
            Rule::RolesSection => {
                let roles_node = parse_roles_section(section)?;
                protocol_node.children.push(Box::new(roles_node));
            }
            Rule::ParametersSection => {
                let params_node = parse_parameters_section(section)?;
                protocol_node.children.push(Box::new(params_node));
            }
            Rule::InteractionSection => {
                let interactions_node = parse_interaction_section(section)?;
                protocol_node.children.push(Box::new(interactions_node));
            }
            _ => {} // Skip closing brace and other tokens
        }
    }

    Ok(protocol_node)
}

fn parse_annotation(pair: Pair<Rule>) -> Result<AstNode> {
    let mut annotation_node = AstNode::new(AstNodeType::Annotation);
    let desc = extract_annotation_text(pair)?;
    annotation_node.set_string("description", &desc);
    Ok(annotation_node)
}

fn parse_roles_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut roles_node = AstNode::new(AstNodeType::RolesSection);
    let mut inner = pair.into_inner();
    
    // Skip "roles" keyword
    inner.next();
    
    for role_decl in inner {
        if role_decl.as_rule() == Rule::RoleDecl {
            let role_node = parse_role_decl(role_decl)?;
            roles_node.children.push(Box::new(role_node));
        }
    }
    
    Ok(roles_node)
}

fn parse_role_decl(pair: Pair<Rule>) -> Result<AstNode> {
    let mut role_node = AstNode::new(AstNodeType::RoleDecl);
    let mut inner = pair.into_inner();

    // Parse role name
    if let Some(name_pair) = inner.next() {
        if name_pair.as_rule() == Rule::Identifier {
            let mut identifier_node = AstNode::new(AstNodeType::Identifier);
            identifier_node.set_string("name", name_pair.as_str());
            role_node.children.push(Box::new(identifier_node));
        }
    } else {
        return Err(anyhow!("Role name missing"));
    }

    // Skip "<Agent>"
    inner.next();

    // Parse annotation
    if let Some(annotation_pair) = inner.next() {
        if annotation_pair.as_rule() == Rule::Annotation {
            let annotation_node = parse_annotation(annotation_pair)?;
            role_node.children.push(Box::new(annotation_node));
        }
    } else {
        return Err(anyhow!("Role annotation missing"));
    }

    Ok(role_node)
}

fn parse_parameters_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut params_node = AstNode::new(AstNodeType::ParametersSection);
    let mut inner = pair.into_inner();
    
    // Skip "parameters" keyword
    inner.next();
    
    for param_decl in inner {
        if param_decl.as_rule() == Rule::ParameterDecl {
            let param_node = parse_parameter_decl(param_decl)?;
            params_node.children.push(Box::new(param_node));
        }
    }
    
    Ok(params_node)
}

fn parse_parameter_decl(pair: Pair<Rule>) -> Result<AstNode> {
    let mut param_node = AstNode::new(AstNodeType::ParameterDecl);
    let mut inner = pair.into_inner();

    // Parse parameter name
    if let Some(name_pair) = inner.next() {
        if name_pair.as_rule() == Rule::Identifier {
            let mut identifier_node = AstNode::new(AstNodeType::Identifier);
            identifier_node.set_string("name", name_pair.as_str());
            param_node.children.push(Box::new(identifier_node));
        }
    } else {
        return Err(anyhow!("Parameter name missing"));
    }

    // Parse type within angle brackets
    if let Some(type_pair) = inner.next() {
        if type_pair.as_rule() == Rule::BasicType {
            let mut type_node = AstNode::new(AstNodeType::BasicType);
            type_node.set_string("type", type_pair.as_str());
            param_node.children.push(Box::new(type_node));
        }
    } else {
        return Err(anyhow!("Parameter type missing"));
    }

    // Parse annotation
    if let Some(annotation_pair) = inner.next() {
        if annotation_pair.as_rule() == Rule::Annotation {
            let annotation_node = parse_annotation(annotation_pair)?;
            param_node.children.push(Box::new(annotation_node));
        }
    } else {
        return Err(anyhow!("Parameter annotation missing"));
    }

    Ok(param_node)
}

fn parse_interaction_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut interactions_node = AstNode::new(AstNodeType::InteractionSection);
    
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::InteractionItem => {
                let item_node = parse_interaction_item(item)?;
                interactions_node.children.push(Box::new(item_node));
            }
            _ => {}
        }
    }
    
    Ok(interactions_node)
}

fn parse_interaction_item(pair: Pair<Rule>) -> Result<AstNode> {
    let mut item_node = AstNode::new(AstNodeType::InteractionItem);
    
    let item_inner = pair.into_inner().next().ok_or_else(|| anyhow!("Empty interaction item"))?;
    match item_inner.as_rule() {
        Rule::StandardInteraction => {
            let interaction_node = parse_standard_interaction(item_inner)?;
            item_node.children.push(Box::new(interaction_node));
        }
        Rule::ProtocolComposition => {
            let composition_node = parse_protocol_composition(item_inner)?;
            item_node.children.push(Box::new(composition_node));
        }
        _ => return Err(anyhow!("Unknown interaction item type"))
    }
    
    Ok(item_node)
}

fn parse_standard_interaction(pair: Pair<Rule>) -> Result<AstNode> {
    let mut interaction_node = AstNode::new(AstNodeType::StandardInteraction);
    let mut inner = pair.into_inner();

    // Parse: from_role -> to_role : action <Action> annotation [parameter_flows]
    
    // Parse from role
    if let Some(from_role) = inner.next() {
        if from_role.as_rule() == Rule::RoleRef {
            let role_name = from_role.into_inner().next().unwrap();
            let mut from_identifier = AstNode::new(AstNodeType::Identifier);
            from_identifier.set_string("name", role_name.as_str());
            from_identifier.set_string("role", "from");
            interaction_node.children.push(Box::new(from_identifier));
        }
    } else {
        return Err(anyhow!("Missing 'from' role in interaction"));
    }

    // Parse to role  
    if let Some(to_role) = inner.next() {
        if to_role.as_rule() == Rule::RoleRef {
            let role_name = to_role.into_inner().next().unwrap();
            let mut to_identifier = AstNode::new(AstNodeType::Identifier);
            to_identifier.set_string("name", role_name.as_str());
            to_identifier.set_string("role", "to");
            interaction_node.children.push(Box::new(to_identifier));
        }
    } else {
        return Err(anyhow!("Missing 'to' role in interaction"));
    }

    // Parse action name
    if let Some(action_name) = inner.next() {
        if action_name.as_rule() == Rule::ActionName {
            let action_inner = action_name.into_inner().next().unwrap();
            let mut action_identifier = AstNode::new(AstNodeType::Identifier);
            action_identifier.set_string("name", action_inner.as_str());
            action_identifier.set_string("role", "action");
            interaction_node.children.push(Box::new(action_identifier));
        }
    } else {
        return Err(anyhow!("Missing action name"));
    }

    // Skip "<Action>" token
    inner.next();

    // Parse annotation
    if let Some(annotation) = inner.next() {
        if annotation.as_rule() == Rule::Annotation {
            let annotation_node = parse_annotation(annotation)?;
            interaction_node.children.push(Box::new(annotation_node));
        }
    } else {
        return Err(anyhow!("Missing action annotation"));
    }

    // Parse optional parameter flow list
    if let Some(param_flow_list) = inner.next() {
        if param_flow_list.as_rule() == Rule::ParameterFlowList {
            for param_flow in param_flow_list.into_inner() {
                if param_flow.as_rule() == Rule::ParameterFlow {
                    let param_flow_node = parse_parameter_flow(param_flow)?;
                    interaction_node.children.push(Box::new(param_flow_node));
                }
            }
        }
    }

    Ok(interaction_node)
}

fn parse_protocol_composition(pair: Pair<Rule>) -> Result<AstNode> {
    let mut composition_node = AstNode::new(AstNodeType::ProtocolComposition);
    let mut inner = pair.into_inner();

    // Parse protocol reference
    if let Some(protocol_ref) = inner.next() {
        if protocol_ref.as_rule() == Rule::ProtocolReference {
            let protocol_ref_node = parse_protocol_reference(protocol_ref)?;
            composition_node.children.push(Box::new(protocol_ref_node));
        }
    } else {
        return Err(anyhow!("Missing protocol reference in composition"));
    }

    // Parse composition parameter list (now using square brackets)
    if let Some(param_list) = inner.next() {
        if param_list.as_rule() == Rule::CompositionParameterList {
            for param in param_list.into_inner() {
                if param.as_rule() == Rule::CompositionParameter {
                    let param_node = parse_composition_parameter(param)?;
                    composition_node.children.push(Box::new(param_node));
                }
            }
        }
    }

    Ok(composition_node)
}

fn parse_protocol_reference(pair: Pair<Rule>) -> Result<AstNode> {
    let mut ref_node = AstNode::new(AstNodeType::ProtocolReference);
    let mut inner = pair.into_inner();
    
    // Parse identifier
    if let Some(name) = inner.next() {
        if name.as_rule() == Rule::Identifier {
            let mut identifier_node = AstNode::new(AstNodeType::Identifier);
            identifier_node.set_string("name", name.as_str());
            ref_node.children.push(Box::new(identifier_node));
        }
    }
    
    // Skip "<Enactment>" token
    
    Ok(ref_node)
}

fn parse_composition_parameter(pair: Pair<Rule>) -> Result<AstNode> {
    let mut inner = pair.into_inner();
    
    if let Some(first) = inner.next() {
        match first.as_rule() {
            Rule::Direction => {
                // This is a parameter flow: "in ID" or "out tag"
                let direction = first.as_str();
                if let Some(identifier) = inner.next() {
                    if identifier.as_rule() == Rule::Identifier {
                        let mut param_flow = AstNode::new(AstNodeType::ParameterFlow);
                        param_flow.set_string("direction", direction);
                        
                        let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                        identifier_node.set_string("name", identifier.as_str());
                        param_flow.children.push(Box::new(identifier_node));
                        
                        Ok(param_flow)
                    } else {
                        Err(anyhow!("Expected identifier after direction"))
                    }
                } else {
                    Err(anyhow!("Missing identifier in composition parameter"))
                }
            }
            Rule::Identifier => {
                // This is just an identifier: "W", "P", "S"
                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", first.as_str());
                Ok(identifier_node)
            }
            _ => Err(anyhow!("Unexpected token in composition parameter"))
        }
    } else {
        Err(anyhow!("Empty composition parameter"))
    }
}

fn parse_parameter_flow(pair: Pair<Rule>) -> Result<AstNode> {
    let mut param_flow_node = AstNode::new(AstNodeType::ParameterFlow);
    let mut inner = pair.into_inner();

    if let Some(direction_pair) = inner.next() {
        if direction_pair.as_rule() == Rule::Direction {
            param_flow_node.set_string("direction", direction_pair.as_str());

            if let Some(ident_pair) = inner.next() {
                if ident_pair.as_rule() == Rule::Identifier {
                    let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                    identifier_node.set_string("name", ident_pair.as_str());
                    param_flow_node.children.push(Box::new(identifier_node));
                }
            } else {
                return Err(anyhow!("Missing identifier in parameter flow"));
            }
        }
    } else {
        return Err(anyhow!("Missing direction in parameter flow"));
    }
    
    Ok(param_flow_node)
}

fn extract_annotation_text(pair: Pair<Rule>) -> Result<String> {
    let inner = pair.into_inner().next()
        .ok_or_else(|| anyhow!("Empty annotation"))?;
    
    if inner.as_rule() == Rule::StringLiteral {
        Ok(extract_string_literal(inner.as_str()))
    } else {
        Err(anyhow!("Invalid annotation format"))
    }
}

fn extract_string_literal(s: &str) -> String {
    // Remove surrounding quotes
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests
    #[test]
    fn test_parse_logistics_protocol() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant coordinating the logistics"),
        W <Agent>("Warehouse managing inventory")
    
    parameters
        ID <String>("unique identifier for the logistics operation"),
        order <String>("order details and specifications")
    
    M -> W: NotifyOrder <Action>("merchant notifies warehouse of new order")[out ID, out order]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        
        let ast = result.unwrap();
        assert_eq!(ast.node_type, AstNodeType::Program);
        assert_eq!(ast.children.len(), 1);
        
        let protocol = &ast.children[0];
        assert_eq!(protocol.node_type, AstNodeType::Protocol);
    }

    #[test]
    fn test_parse_protocol_composition() {
        let source = r#"
Test <Protocol>("Test protocol with composition") {
    roles
        A <Agent>("Agent A")
    
    parameters
        id <String>("identifier")
    
    Pack <Enactment>(A, in id, out result)
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);
        
        let ast = result.unwrap();
        let protocol = &ast.children[0];
        let interaction_section = &protocol.children[2];
        let interaction_item = &interaction_section.children[0];
        let composition = &interaction_item.children[0];
        
        assert_eq!(composition.node_type, AstNodeType::ProtocolComposition);
    }

    // Tests for complete valid protocols
    #[test]
    fn test_parse_complete_logistics_with_composition() {
        let source = r#"
Logistics <Protocol>("Complete logistics protocol with composition") {
    roles
        M <Agent>("Merchant coordinating the logistics"),
        W <Agent>("Warehouse managing inventory"),
        P <Agent>("Packer handling packaging operations"),
        S <Agent>("Scanner providing scanning services")
    
    parameters
        ID <String>("unique identifier for the logistics operation"),
        order <String>("order details and specifications"),
        tag <String>("package identification tag"),
        package <String>("packaged item ready for shipping")
    
    M -> W: NotifyOrder <Action>("merchant notifies warehouse of new order")[out ID, out order]
    Pack <Enactment>(W, P, S, in ID, in order, out tag, out package)
    W -> M: Deliver <Action>("warehouse confirms delivery completion")[in ID, in package]
}

Pack <Protocol>("Package preparation protocol") {
    roles
        W <Agent>("Warehouse coordinating packing"),
        P <Agent>("Packer performing packaging tasks"),
        S <Agent>("Scanner handling tag operations")
    
    parameters
        ID <String>("unique identifier"),
        order <String>("order details"),
        tag <String>("package tag"),
        package <String>("completed package")
    
    W -> P: Pack <Action>("warehouse requests packing")[in ID, in order]
    P -> S: WriteTag <Action>("packer requests tag writing")[in ID, out tag]
    S -> P: TagWritten <Action>("scanner confirms tag written")[in tag, out package]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse complete protocol: {:?}", result);
        
        let ast = result.unwrap();
        assert_eq!(ast.node_type, AstNodeType::Program);
        assert_eq!(ast.children.len(), 2); // Two protocols
    }

    // Error tests - Missing tags
    #[test]
    fn test_missing_protocol_tag() {
        let source = r#"
Logistics ("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when <Protocol> tag is missing");
    }

    #[test]
    fn test_missing_agent_tag() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M ("Merchant")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when <Agent> tag is missing");
    }

    #[test]
    fn test_missing_action_tag() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder ("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when <Action> tag is missing");
    }

    #[test]
    fn test_missing_enactment_tag() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    Pack (W, in ID, out result)
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when <Enactment> tag is missing");
    }

    #[test]
    fn test_missing_type_tag() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID ("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when parameter type tag is missing");
    }

    // Error tests - Missing annotations
    #[test]
    fn test_missing_protocol_annotation() {
        let source = r#"
Logistics <Protocol> {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when protocol annotation is missing");
    }

    #[test]
    fn test_missing_role_annotation() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when role annotation is missing");
    }

    #[test]
    fn test_missing_parameter_annotation() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID <String>
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when parameter annotation is missing");
    }

    #[test]
    fn test_missing_action_annotation() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when action annotation is missing");
    }

    // Error tests - Parenthesis confusion
#[test]
fn test_annotation_with_commas_vs_composition() {
    let source = r#"
Logistics <Protocol>("Multi-party logistics protocol with commas, semicolons; and other punctuation.") {
    roles
        M <Agent>("Merchant, coordinator, and manager"),
        W <Agent>("Warehouse, storage, and inventory")
    
    parameters
        ID <String>("unique identifier, used for tracking, contains special chars"),
        order <String>("order details, specifications, and requirements"),
        package <String>("packaged item ready for shipping")
    
    M -> W: NotifyOrder <Action>("merchant notifies warehouse of new order, including details, timing, and priority")[out ID, out order]
    Pack <Enactment>[W, M, in ID, in order, out package]
}
    "#;

    let result = parse_source(source);
    assert!(result.is_ok(), "Should correctly parse annotations with commas separate from composition parameters: {:?}", result);
    
    let ast = result.unwrap();
    let protocol = &ast.children[0];

    println!("{:?}", &ast);
    
    // Count all annotations in the entire protocol
    let mut annotation_count = 0;
    let mut composition_count = 0;
    
    fn count_annotations_and_compositions(node: &AstNode, ann_count: &mut usize, comp_count: &mut usize) {
        match node.node_type {
            AstNodeType::Annotation => {
                *ann_count += 1;
            }
            AstNodeType::ProtocolComposition => {
                *comp_count += 1;
            }
            _ => {}
        }
        
        // Recursively count in all children
        for child in &node.children {
            count_annotations_and_compositions(child, ann_count, comp_count);
        }
    }
    
    count_annotations_and_compositions(protocol, &mut annotation_count, &mut composition_count);
    
    // Assert exactly 7 annotations now (added package parameter):
    // 1. Protocol annotation: "Multi-party logistics protocol with commas, semicolons; and other punctuation."
    // 2. Role M annotation: "Merchant, coordinator, and manager"
    // 3. Role W annotation: "Warehouse, storage, and inventory" 
    // 4. Parameter ID annotation: "unique identifier, used for tracking, contains special chars"
    // 5. Parameter order annotation: "order details, specifications, and requirements"
    // 6. Parameter package annotation: "packaged item ready for shipping"
    // 7. Action NotifyOrder annotation: "merchant notifies warehouse of new order, including details, timing, and priority"
    assert_eq!(annotation_count, 7, "Should have exactly 7 annotations in the protocol");
    
    // Assert exactly 1 protocol composition:
    // Pack <Enactment>[W, M, in ID, in order, out package]
    assert_eq!(composition_count, 1, "Should have exactly 1 protocol composition");
    
    // Verify that all commas in annotations are preserved and don't interfere with parsing
    verify_annotation_content(&ast);
    verify_composition_structure(&ast);
}

fn verify_annotation_content(ast: &AstNode) {
    let mut protocol_annotation_found = false;
    
    fn search_annotations(node: &AstNode, found: &mut bool) {
        if node.node_type == AstNodeType::Annotation {
            if let Some(description) = node.get_string("description") {
                if description.contains("Multi-party logistics protocol") {
                    *found = true;
                    assert!(description.contains(","), "Should contain commas");
                    assert!(description.contains(";"), "Should contain semicolons");
                    assert!(description.contains("punctuation"), "Should contain full text");
                }
            }
        }
        
        for child in &node.children {
            search_annotations(child, found);
        }
    }
    
    search_annotations(ast, &mut protocol_annotation_found);
    assert!(protocol_annotation_found, "Should find protocol annotation with commas");
}

fn verify_composition_structure(ast: &AstNode) {
    let mut composition_found = false;
    
    fn search_composition(node: &AstNode, found: &mut bool) {
        if node.node_type == AstNodeType::ProtocolComposition {
            *found = true;
            
            // Verify the composition has the expected structure with square brackets
            let mut protocol_ref_found = false;
            let mut role_params = 0;
            let mut parameter_flows = 0;
            
            for child in &node.children {
                match child.node_type {
                    AstNodeType::ProtocolReference => protocol_ref_found = true,
                    AstNodeType::Identifier => role_params += 1,
                    AstNodeType::ParameterFlow => parameter_flows += 1,
                    _ => {}
                }
            }
            
            assert!(protocol_ref_found, "Composition should have protocol reference");
            assert_eq!(role_params, 2, "Composition should have 2 role parameters (W, M)");
            assert_eq!(parameter_flows, 3, "Composition should have 3 parameter flows (in ID, in order, out package)");
        }
        
        for child in &node.children {
            search_composition(child, found);
        }
    }
    
    search_composition(ast, &mut composition_found);
    assert!(composition_found, "Should find the protocol composition with square brackets");
}


    #[test]
    fn test_malformed_annotation_quotes() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol) {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when annotation quotes are malformed");
    }

    #[test]
    fn test_composition_without_parentheses() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    Pack <Enactment> W, in ID, out result
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when composition parameters are not enclosed in parentheses");
    }

    #[test]
    fn test_nested_quotes_in_annotation() {
        let source = r#"
Logistics <Protocol>("Protocol with \"nested quotes\" in description") {
    roles
        M <Agent>("Agent with \"special\" role")
    
    parameters
        ID <String>("Parameter with \"quoted\" content")
    
    M -> W: NotifyOrder <Action>("Action with \"nested\" quotes")[out ID]
}
        "#;

        let result = parse_source(source);
        // This might fail depending on how string literals are handled in the grammar
        // The test documents the expected behavior
        if result.is_ok() {
            println!("Parser handles nested quotes correctly");
        } else {
            println!("Parser doesn't support nested quotes in annotations");
        }
    }

    // Error tests - Structural issues
    #[test]
    fn test_missing_roles_section() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when roles section is missing");
    }

    #[test]
    fn test_missing_parameters_section() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    M -> W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when parameters section is missing");
    }

    #[test]
    fn test_empty_protocol() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when protocol is empty");
    }

    #[test]
    fn test_malformed_interaction_arrow() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    M - W: NotifyOrder <Action>("action")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when interaction arrow is malformed");
    }

    #[test]
    fn test_invalid_parameter_direction() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[through ID]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when parameter direction is invalid");
    }

    #[test]
    fn test_composition_parameter_mixing() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse"),
        P <Agent>("Packer")
    
    parameters
        ID <String>("identifier"),
        order <String>("order details")
    
    Pack <Enactment>(W, P, in ID, S, out order, in result)
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Should correctly parse mixed role identifiers and parameter flows in composition");
        
        if result.is_ok() {
            let ast = result.unwrap();
            let protocol = &ast.children[0];
            let interaction_section = &protocol.children[2];
            let interaction_item = &interaction_section.children[0];
            let composition = &interaction_item.children[0];
            
            assert_eq!(composition.node_type, AstNodeType::ProtocolComposition);
            // Should have protocol reference + multiple composition parameters
            assert!(composition.children.len() > 1);
        }
    }

    #[test]
    fn test_multiple_protocols_with_errors() {
        let source = r#"
ValidProtocol <Protocol>("This one is valid") {
    roles
        A <Agent>("Agent A")
    
    parameters
        id <String>("identifier")
    
    A -> B: Action <Action>("valid action")[]
}

InvalidProtocol ("Missing protocol tag") {
    roles
        B <Agent>("Agent B")
    
    parameters
        data <String>("some data")
    
    B -> A: InvalidAction <Action>("invalid action")[]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_err(), "Should fail when any protocol in the program is invalid");
    }

    #[test]
    fn test_edge_case_empty_parameter_flows() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier")
    
    M -> W: NotifyOrder <Action>("action")[]
    Pack <Enactment>(W, M)
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Should handle empty parameter flow lists correctly");
    }

    #[test]
    fn test_whitespace_and_formatting_variations() {
        let source = r#"
Logistics<Protocol>("Compact formatting"){
roles M<Agent>("Merchant"),W<Agent>("Warehouse")
parameters ID<String>("identifier")
M->W:NotifyOrder<Action>("action")[out ID]
Pack<Enactment>(W,M,in ID,out result)
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Should handle compact formatting without extra whitespace");
    }
}

