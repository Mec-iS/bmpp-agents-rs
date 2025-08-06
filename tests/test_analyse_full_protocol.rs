#[cfg(test)]
mod tests {
    use bmpp_agents::transpiler::parser::parse_source;
    use bmpp_agents::protocol::ast::{AstNodeType};
    use anyhow::Result;
    use std::fs;
    use std::path::Path;

    fn load_protocol_file(filename: &str) -> Result<String> {
        let file_path = Path::new("examples").join(filename);
        fs::read_to_string(&file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read protocol file {}: {}", file_path.display(), e))
    }

    #[test]
    fn test_analyze_protocol_usage_example() -> Result<()> {
        let source = load_protocol_file("STARTER-PROTOCOL_COMPOSED.bmpp")?;

        let ast = parse_source(&source)?;
        
        println!("Complete AST Structure:");
        println!("{}", ast);
        
        println!("\nAnalyzing individual protocols:");
        for protocol in &ast.children {
            if protocol.node_type == AstNodeType::Protocol {
                let name = protocol.get_protocol_name().unwrap_or("Unknown".to_string());
                println!("\nProtocol: {}", name);
                println!("{}", protocol);
            }
        }

        Ok(())
    }

    #[test]
    fn test_analyze_protocol_output_validation() -> Result<()> {
        let source = r#"
TestProtocol <Protocol>("A test protocol for validation") {
    roles
        Producer <Agent>("Produces data"),
        Consumer <Agent>("Consumes data")
    
    parameters
        data <String>("The data to be transferred"),
        result <Bool>("Processing result")
    
    Producer -> Consumer: transfer <Action>("Transfer data")[out data]
    Consumer -> Producer: acknowledge <Action>("Acknowledge receipt")[in data, out result]
}
        "#;

        let ast = parse_source(source)?;
        
        println!("Test Protocol AST:");
        println!("{}", ast);
        
        for protocol in &ast.children {
            if protocol.node_type == AstNodeType::Protocol {
                let name = protocol.get_protocol_name();
                assert_eq!(name, Some("TestProtocol".to_string()));
                
                if let Some(roles_section) = protocol.get_roles_section() {
                    let roles = roles_section.get_role_declarations();
                    assert_eq!(roles.len(), 2);
                    
                    let role_info: Vec<_> = roles.iter()
                        .filter_map(|r| r.get_role_decl_info())
                        .collect();
                    
                    assert!(role_info.contains(&("Producer".to_string(), "Produces data".to_string())));
                    assert!(role_info.contains(&("Consumer".to_string(), "Consumes data".to_string())));
                }
                
                if let Some(params_section) = protocol.get_parameters_section() {
                    let params = params_section.get_parameter_declarations();
                    assert_eq!(params.len(), 2);
                    
                    let param_info: Vec<_> = params.iter()
                        .filter_map(|p| p.get_parameter_decl_info())
                        .collect();
                    
                    assert!(param_info.contains(&("data".to_string(), "String".to_string(), "The data to be transferred".to_string())));
                    assert!(param_info.contains(&("result".to_string(), "Bool".to_string(), "Processing result".to_string())));
                }
                
                if let Some(interactions_section) = protocol.get_interactions_section() {
                    let items = interactions_section.get_interaction_items();
                    assert_eq!(items.len(), 2);
                    
                    let first_interaction = items[0].get_standard_interaction().unwrap();
                    let info = first_interaction.get_standard_interaction_info().unwrap();
                    
                    assert_eq!(info.from_role, "Producer");
                    assert_eq!(info.to_role, "Consumer");
                    assert_eq!(info.action_name, "transfer");
                    assert_eq!(info.parameter_flows.len(), 1);
                    assert_eq!(info.parameter_flows[0], ("out".to_string(), "data".to_string()));
                }
            }
        }
        
        println!("✅ All analyze_protocol validations passed");
        Ok(())
    }

    #[test]
    fn test_analyze_protocol_with_compositions() -> Result<()> {
        let source = r#"
MainProtocol <Protocol>("Protocol with compositions") {
    roles
        A <Agent>("Agent A"),
        B <Agent>("Agent B"),
        C <Agent>("Agent C")
    
    parameters
        input <String>("Input data"),
        output <String>("Output data")
    
    A -> B: start <Action>("Start process")[out input]
    SubProcess <Enactment>[B, C, in input, out output]
    B -> A: finish <Action>("Finish process")[in output]
}

SubProcess <Protocol>("Sub-process protocol") {
    roles
        X <Agent>("Agent X"),
        Y <Agent>("Agent Y")
    
    parameters
        data <String>("Data to process"),
        result <String>("Processed result")
    
    X -> Y: process <Action>("Process data")[in data, out result]
}
        "#;

        let ast = parse_source(source)?;
        
        println!("Protocol with Compositions AST:");
        println!("{}", ast);
        
        for protocol in &ast.children {
            if protocol.node_type == AstNodeType::Protocol {
                let name = protocol.get_protocol_name().unwrap_or_default();
                
                if name == "MainProtocol" {
                    if let Some(interactions_section) = protocol.get_interactions_section() {
                        let items = interactions_section.get_interaction_items();
                        
                        assert_eq!(items.len(), 3);
                        
                        let composition_item = &items[1];
                        let composition = composition_item.get_protocol_composition();
                        assert!(composition.is_some(), "Should find protocol composition");
                        
                        let comp = composition.unwrap();
                        let protocol_ref = comp.get_protocol_reference().unwrap();
                        let ref_name = protocol_ref.get_identifier_name().unwrap();
                        assert_eq!(ref_name, "SubProcess");
                        
                        let param_flows = comp.get_parameter_flows();
                        assert_eq!(param_flows.len(), 2);
                        
                        let flows: Vec<_> = param_flows.iter()
                            .filter_map(|pf| pf.get_parameter_flow_info())
                            .collect();
                        
                        assert!(flows.contains(&("in".to_string(), "input".to_string())));
                        assert!(flows.contains(&("out".to_string(), "output".to_string())));
                    }
                }
            }
        }
        
        println!("✅ Protocol composition analysis test passed");
        Ok(())
    }

    #[test]
    fn test_display_specific_sections() -> Result<()> {
        let source = r#"
DisplayTest <Protocol>("Testing Display trait on sections") {
    roles
        TestRole <Agent>("A test role")
    
    parameters
        testParam <String>("A test parameter")
    
    TestRole -> TestRole: selfAction <Action>("Self action")[out testParam, in testParam]
}
        "#;

        let ast = parse_source(source)?;
        
        for protocol in &ast.children {
            if protocol.node_type == AstNodeType::Protocol {
                println!("Full Protocol Display:");
                println!("{}", protocol);
                
                if let Some(roles_section) = protocol.get_roles_section() {
                    println!("\nRoles Section Only:");
                    println!("{}", roles_section);
                }
                
                if let Some(params_section) = protocol.get_parameters_section() {
                    println!("\nParameters Section Only:");
                    println!("{}", params_section);
                }
                
                if let Some(interactions_section) = protocol.get_interactions_section() {
                    println!("\nInteractions Section Only:");
                    println!("{}", interactions_section);
                    
                    for item in interactions_section.get_interaction_items() {
                        println!("\nInteraction Item:");
                        println!("{}", item);
                    }
                }
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_loaded_file_protocol_validation() -> Result<()> {
        let source = load_protocol_file("STARTER-PROTOCOL_COMPOSED.bmpp")?;
        let ast = parse_source(&source)?;
        
        // Validate that the file was loaded and parsed successfully
        assert_eq!(ast.node_type, AstNodeType::Program);
        assert!(!ast.children.is_empty(), "Program should contain at least one protocol");
        
        let protocol_names: Vec<String> = ast.children.iter()
            .filter(|child| child.node_type == AstNodeType::Protocol)
            .filter_map(|protocol| protocol.get_protocol_name())
            .collect();
        
        println!("Found protocols: {:?}", protocol_names);
        assert!(!protocol_names.is_empty(), "Should find at least one protocol");
        
        // Validate structure of each protocol
        for protocol in &ast.children {
            if protocol.node_type == AstNodeType::Protocol {
                let name = protocol.get_protocol_name().unwrap_or("Unknown".to_string());
                
                assert!(protocol.get_roles_section().is_some(), "Protocol {} should have roles section", name);
                assert!(protocol.get_parameters_section().is_some(), "Protocol {} should have parameters section", name);
                assert!(protocol.get_interactions_section().is_some(), "Protocol {} should have interactions section", name);
            }
        }
        
        println!("✅ Loaded protocol file validation passed");
        Ok(())
    }
}
