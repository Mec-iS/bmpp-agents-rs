use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::{Result, anyhow};
use pest::Parser;
use pest::iterators::Pair;

#[derive(pest_derive::Parser)]
#[grammar = "src/grammars/v1/bmpp.pest"]
pub struct BmppParser;

pub fn parse_source(source: &str) -> Result<AstNode> {
    let pairs = BmppParser::parse(Rule::Program, source)?
        .next()
        .ok_or_else(|| anyhow!("Failed to parse program: no pairs found"))?;

    let mut program_node = AstNode::new(AstNodeType::Program);

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::Protocol => {
                let protocol_node = build_ast_from_pair(pair)?;
                program_node.add_child(protocol_node);
            }
            Rule::EOI => {} // End of input, ignore
            _ => return Err(anyhow!("Unexpected rule in program: {:?}", pair.as_rule())),
        }
    }

    Ok(program_node)
}

fn build_ast_from_pair(pair: Pair<Rule>) -> Result<AstNode> {
    match pair.as_rule() {
        Rule::Protocol => {
            let mut inner = pair.into_inner();

            // Parse protocol name
            let name_pair = inner.next().unwrap();
            let name = name_pair.as_str();

            // Parse protocol annotation (includes parentheses)
            let annotation_pair = inner.next().unwrap();
            let description = parse_annotation(annotation_pair)?;

            let mut protocol_node = AstNode::new(AstNodeType::ProtocolDecl);
            protocol_node.set_string("name", name);
            protocol_node.set_string("description", &description);

            // Parse mandatory sections in order
            let roles_section = inner
                .next()
                .ok_or_else(|| anyhow!("Missing mandatory roles section"))?;
            if roles_section.as_rule() != Rule::RolesSection {
                return Err(anyhow!(
                    "Expected roles section, found: {:?}",
                    roles_section.as_rule()
                ));
            }
            let roles_node = build_roles_section(roles_section)?;
            protocol_node.add_child(roles_node);

            let params_section = inner
                .next()
                .ok_or_else(|| anyhow!("Missing mandatory parameters section"))?;
            if params_section.as_rule() != Rule::ParametersSection {
                return Err(anyhow!(
                    "Expected parameters section, found: {:?}",
                    params_section.as_rule()
                ));
            }
            let params_node = build_parameters_section(params_section)?;
            protocol_node.add_child(params_node);

            let interactions_section = inner
                .next()
                .ok_or_else(|| anyhow!("Missing mandatory interactions section"))?;
            if interactions_section.as_rule() != Rule::InteractionSection {
                return Err(anyhow!(
                    "Expected interactions section, found: {:?}",
                    interactions_section.as_rule()
                ));
            }
            let interactions_node = build_interactions_section(interactions_section)?;
            protocol_node.add_child(interactions_node);

            Ok(protocol_node)
        }
        _ => Err(anyhow!("Unhandled grammar rule: {:?}", pair.as_rule())),
    }
}

fn build_roles_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut roles_node = AstNode::new(AstNodeType::RolesSection);
    let mut role_count = 0;

    for role_pair in pair.into_inner() {
        match role_pair.as_rule() {
            Rule::RoleDecl => {
                let mut inner = role_pair.into_inner();
                let role_name = inner.next().unwrap().as_str();

                // Validate Agent type is present (implicitly validated by grammar)
                let annotation_pair = inner.next().unwrap();
                let description = parse_annotation(annotation_pair)?;

                let mut role_node = AstNode::new(AstNodeType::RoleDecl);
                role_node.set_string("name", role_name);
                role_node.set_string("type", "Agent");
                role_node.set_string("description", &description);

                roles_node.add_child(role_node);
                role_count += 1;
            }
            _ => {
                return Err(anyhow!(
                    "Unexpected rule in roles section: {:?}",
                    role_pair.as_rule()
                ));
            }
        }
    }

    if role_count == 0 {
        return Err(anyhow!("Roles section cannot be empty"));
    }

    Ok(roles_node)
}

fn build_parameters_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut params_node = AstNode::new(AstNodeType::ParametersSection);
    let mut param_count = 0;

    for param_pair in pair.into_inner() {
        match param_pair.as_rule() {
            Rule::ParameterDecl => {
                let mut inner = param_pair.into_inner();
                let param_name = inner.next().unwrap().as_str();
                let param_type = inner.next().unwrap().as_str();

                // Validate type is a BasicType (implicitly validated by grammar)
                if !matches!(param_type, "String" | "Int" | "Float" | "Bool") {
                    return Err(anyhow!("Invalid parameter type: {}", param_type));
                }

                let annotation_pair = inner.next().unwrap();
                let description = parse_annotation(annotation_pair)?;

                let mut param_node = AstNode::new(AstNodeType::ParameterDecl);
                param_node.set_string("name", param_name);
                param_node.set_string("type", param_type);
                param_node.set_string("description", &description);

                params_node.add_child(param_node);
                param_count += 1;
            }
            _ => {
                return Err(anyhow!(
                    "Unexpected rule in parameters section: {:?}",
                    param_pair.as_rule()
                ));
            }
        }
    }

    if param_count == 0 {
        return Err(anyhow!("Parameters section cannot be empty"));
    }

    Ok(params_node)
}

fn build_interactions_section(pair: Pair<Rule>) -> Result<AstNode> {
    let mut interactions_node = AstNode::new(AstNodeType::InteractionsSection);
    let mut interaction_count = 0;

    for interaction_pair in pair.into_inner() {
        match interaction_pair.as_rule() {
            Rule::Interaction => {
                let interaction_node = build_interaction(interaction_pair)?;
                interactions_node.add_child(interaction_node);
                interaction_count += 1;
            }
            _ => {
                return Err(anyhow!(
                    "Unexpected rule in interactions section: {:?}",
                    interaction_pair.as_rule()
                ));
            }
        }
    }

    if interaction_count == 0 {
        return Err(anyhow!("Interactions section cannot be empty"));
    }

    Ok(interactions_node)
}

fn build_interaction(pair: Pair<Rule>) -> Result<AstNode> {
    let mut inner = pair.into_inner();

    let from_role = inner.next().unwrap().as_str();
    let to_role = inner.next().unwrap().as_str();
    let action_name = inner.next().unwrap().as_str();

    // Parse annotation (mandatory)
    let annotation_pair = inner.next().unwrap();
    let description = parse_annotation(annotation_pair)?;

    let mut interaction_node = AstNode::new(AstNodeType::InteractionDecl);
    interaction_node.set_string("from_role", from_role);
    interaction_node.set_string("to_role", to_role);
    interaction_node.set_string("action", action_name);
    interaction_node.set_string("description", &description);

    // Parse parameter flow (mandatory brackets, but content can be empty)
    if let Some(param_flow_pair) = inner.next() {
        let param_flow_node = build_parameter_flow(param_flow_pair)?;
        interaction_node.add_child(param_flow_node);
    }

    Ok(interaction_node)
}

fn build_parameter_flow(pair: Pair<Rule>) -> Result<AstNode> {
    let mut flow_node = AstNode::new(AstNodeType::ParameterFlow);

    for param_flow_pair in pair.into_inner() {
        match param_flow_pair.as_rule() {
            Rule::ParameterFlow => {
                let mut inner = param_flow_pair.into_inner();
                let direction = inner.next().unwrap().as_str();
                let param_name = inner.next().unwrap().as_str();

                // Validate direction
                if !matches!(direction, "in" | "out") {
                    return Err(anyhow!("Invalid parameter direction: {}", direction));
                }

                let mut param_ref_node = AstNode::new(AstNodeType::ParameterRef);
                param_ref_node.set_string("direction", direction);
                param_ref_node.set_string("parameter", param_name);

                flow_node.add_child(param_ref_node);
            }
            _ => {
                return Err(anyhow!(
                    "Unexpected rule in parameter flow: {:?}",
                    param_flow_pair.as_rule()
                ));
            }
        }
    }

    Ok(flow_node)
}

fn parse_annotation(pair: Pair<Rule>) -> Result<String> {
    if pair.as_rule() != Rule::Annotation {
        return Err(anyhow!("Expected annotation, found: {:?}", pair.as_rule()));
    }

    let mut inner = pair.into_inner();
    let string_literal = inner.next().unwrap();

    if string_literal.as_rule() != Rule::StringLiteral {
        return Err(anyhow!("Expected string literal in annotation"));
    }

    parse_string_literal(string_literal.as_str())
}

fn parse_string_literal(s: &str) -> Result<String> {
    if !s.starts_with('"') || !s.ends_with('"') {
        return Err(anyhow!("Invalid string literal format: {}", s));
    }

    // Remove surrounding quotes and handle escape sequences
    Ok(s[1..s.len() - 1]
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_complete_protocol() -> Result<()> {
        let source = r#"
        Purchase <Protocol>("the generic action of acquiring a generic item") {
            roles
                B <Agent>("the party wanting to buy an item"),
                S <Agent>("the party selling the item")
            
            parameters
                ID <String>("a unique identifier for the request"),
                item <String>("the name or description of the product")
            
            B -> S: rfq <Action>("request for a price quote")[out ID, out item]
            S -> B: quote <Action>("provide a price quote")[in ID, in item]
        }
        "#;

        let ast = parse_source(source)?;
        assert_eq!(ast.node_type, AstNodeType::Program);
        assert_eq!(ast.children.len(), 1);

        let protocol_node = &ast.children[0];
        assert_eq!(protocol_node.node_type, AstNodeType::ProtocolDecl);
        assert_eq!(protocol_node.get_string("name").unwrap(), "Purchase");
        assert_eq!(
            protocol_node.get_string("description").unwrap(),
            "the generic action of acquiring a generic item"
        );

        // Verify all mandatory sections are present
        assert_eq!(protocol_node.children.len(), 3);
        assert_eq!(
            protocol_node.children[0].node_type,
            AstNodeType::RolesSection
        );
        assert_eq!(
            protocol_node.children[1].node_type,
            AstNodeType::ParametersSection
        );
        assert_eq!(
            protocol_node.children[2].node_type,
            AstNodeType::InteractionsSection
        );

        Ok(())
    }

    #[test]
    fn test_mandatory_annotations_with_parentheses() -> Result<()> {
        let source = r#"
        Test <Protocol>("test protocol") {
            roles
                A <Agent>("test agent")
            
            parameters
                id <String>("test parameter")
            
            A -> A: test <Action>("test action")[out id]
        }
        "#;

        let ast = parse_source(source)?;
        let protocol = &ast.children[0];

        // Check protocol annotation
        assert_eq!(protocol.get_string("description").unwrap(), "test protocol");

        // Check role annotation
        let roles = &protocol.children[0];
        let role = &roles.children[0];
        assert_eq!(role.get_string("description").unwrap(), "test agent");

        // Check parameter annotation
        let params = &protocol.children[1];
        let param = &params.children[0];
        assert_eq!(param.get_string("description").unwrap(), "test parameter");

        // Check interaction annotation
        let interactions = &protocol.children[2];
        let interaction = &interactions.children[0];
        assert_eq!(
            interaction.get_string("description").unwrap(),
            "test action"
        );

        Ok(())
    }

    #[test]
    fn test_missing_parentheses_in_annotation_fails() {
        let source = r#"
        Test <Protocol> "missing parentheses" {
            roles A <Agent>("test")
            parameters id <String>("test")
            A -> A: test <Action>("test")[out id]
        }
        "#;

        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_type_annotation_fails() {
        let source = r#"
        Test <Protocol>("test") {
            roles A ("missing type annotation")
            parameters id <String>("test")
            A -> A: test <Action>("test")[out id]
        }
        "#;

        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_basic_type_fails() {
        let source = r#"
        Test <Protocol>("test") {
            roles A <Agent>("test")
            parameters id <InvalidType>("test parameter")
            A -> A: test <Action>("test")[out id]
        }
        "#;

        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_sections_fail() {
        let source = r#"
        Test <Protocol>("test") {
            roles
            parameters id <String>("test")
            A -> A: test <Action>("test")[out id]
        }
        "#;

        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_mandatory_sections_fail() {
        // Missing parameters section
        let source = r#"
        Test <Protocol>("test") {
            roles A <Agent>("test")
            A -> A: test <Action>("test")[]
        }
        "#;

        let result = parse_source(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_flow_directions() -> Result<()> {
        let source = r#"
        Test <Protocol>("test") {
            roles
                A <Agent>("test agent"),
                B <Agent>("another agent")
            
            parameters
                input <String>("input parameter"),
                output <String>("output parameter")
            
            A -> B: transfer <Action>("transfer data")[in input, out output]
        }
        "#;

        let ast = parse_source(source)?;
        let protocol = &ast.children[0];
        let interactions = &protocol.children[2];
        let interaction = &interactions.children[0];
        let param_flow = &interaction.children[0];

        assert_eq!(param_flow.children.len(), 2);

        let in_param = &param_flow.children[0];
        assert_eq!(in_param.get_string("direction").unwrap(), "in");
        assert_eq!(in_param.get_string("parameter").unwrap(), "input");

        let out_param = &param_flow.children[1];
        assert_eq!(out_param.get_string("direction").unwrap(), "out");
        assert_eq!(out_param.get_string("parameter").unwrap(), "output");

        Ok(())
    }

    #[test]
    fn test_empty_parameter_flow() -> Result<()> {
        let source = r#"
        Test <Protocol>("test") {
            roles A <Agent>("test")
            parameters dummy <String>("dummy")
            A -> A: noop <Action>("no operation")[]
        }
        "#;

        let ast = parse_source(source)?;
        let protocol = &ast.children[0];
        let interactions = &protocol.children[2];
        let interaction = &interactions.children[0];

        // Should have parameter flow node even if empty
        assert_eq!(interaction.children.len(), 1);
        let param_flow = &interaction.children[0];
        assert_eq!(param_flow.children.len(), 0);

        Ok(())
    }

    #[test]
    fn test_all_basic_types() -> Result<()> {
        let source = r#"
        Test <Protocol>("test all types") {
            roles A <Agent>("test agent")
            
            parameters
                str_param <String>("string parameter"),
                int_param <Int>("integer parameter"),
                float_param <Float>("float parameter"),
                bool_param <Bool>("boolean parameter")
            
            A -> A: test <Action>("test action")[out str_param, out int_param, out float_param, out bool_param]
        }
        "#;

        let ast = parse_source(source)?;
        let protocol = &ast.children[0];
        let params_section = &protocol.children[1];

        assert_eq!(params_section.children.len(), 4);

        let types = vec!["String", "Int", "Float", "Bool"];
        for (i, expected_type) in types.iter().enumerate() {
            let param = &params_section.children[i];
            assert_eq!(param.get_string("type").unwrap(), expected_type);
        }

        Ok(())
    }

    #[test]
    fn test_multiple_roles_and_interactions() -> Result<()> {
        let source = r#"
        MultiParty <Protocol>("multi-party protocol") {
            roles
                A <Agent>("first agent"),
                B <Agent>("second agent"),
                C <Agent>("third agent")
            
            parameters
                data <String>("shared data"),
                result <Bool>("operation result")
            
            A -> B: send <Action>("send data")[out data]
            B -> C: forward <Action>("forward data")[in data, out result]
            C -> A: complete <Action>("complete operation")[in result]
        }
        "#;

        let ast = parse_source(source)?;
        let protocol = &ast.children[0];

        // Check roles
        let roles_section = &protocol.children[0];
        assert_eq!(roles_section.children.len(), 3);

        // Check interactions
        let interactions_section = &protocol.children[2];
        assert_eq!(interactions_section.children.len(), 3);

        let interaction_names = vec!["send", "forward", "complete"];
        for (i, expected_name) in interaction_names.iter().enumerate() {
            let interaction = &interactions_section.children[i];
            assert_eq!(interaction.get_string("action").unwrap(), expected_name);
        }

        Ok(())
    }

    #[test]
    fn test_comments_are_ignored() -> Result<()> {
        let source = r#"
        // This is a line comment
        Test <Protocol>("test with comments") {
            // Another comment
            roles
                A <Agent>("test agent") // trailing comment
            
            /* Block comment */
            parameters
                id <String>("identifier") /* inline block comment */
            
            // Comment before interaction
            A -> A: test <Action>("test action")[out id]
        }
        "#;

        let ast = parse_source(source)?;
        assert_eq!(ast.children.len(), 1);

        let protocol = &ast.children[0];
        assert_eq!(protocol.get_string("name").unwrap(), "Test");

        Ok(())
    }
}
