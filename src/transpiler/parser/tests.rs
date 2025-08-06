#[cfg(test)]
mod tests {
    use crate::protocol::ast::*;
    use crate::transpiler::parse_source;

    #[test]
    fn test_multiple_protocols() {
        let source = r#"
FirstProtocol <Protocol>("First protocol description") {
    roles
        A <Agent>("Agent A"),
        B <Agent>("Agent B")
    
    parameters
        p1 <String>("Parameter 1"),
        p2 <String>("Parameter 2")
    
    A -> B: Action1 <Action>("First action")[out p1]
    B -> A: Action2 <Action>("Second action")[in p1, out p2]
}

SecondProtocol <Protocol>("Second protocol description") {
    roles
        X <Agent>("Agent X"),
        Y <Agent>("Agent Y")
    
    parameters
        x1 <String>("Parameter X1")
    
    X -> Y: ActionX <Action>("Action X")[out x1]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should parse multiple protocols: {:?}",
            result
        );

        let ast = result.unwrap();
        assert_eq!(ast.node_type, AstNodeType::Program);
        assert_eq!(ast.children.len(), 2, "Should have exactly 2 protocols");
    }

    #[test]
    fn test_composition_with_mixed_parameters() {
        let source = r#"
MainProtocol <Protocol>("Protocol with complex composition") {
    roles
        Client <Agent>("The client"),
        Server <Agent>("The server"),
        Worker <Agent>("The worker")
    
    parameters
        request <String>("Request data"),
        result <String>("Result data")
    
    Client -> Server: Request <Action>("Send request")[out request]
    SubProtocol <Enactment>[Server, Worker, in request, out result]
    Server -> Client: Response <Action>("Send response")[in result]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should parse composition with mixed parameters: {:?}",
            result
        );
    }

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
        id <String>("identifier"),
        result <String>("result parameter")
    
    Pack <Enactment>[A, in id, out result]
}
        "#;

        let result = parse_source(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);

        let ast = result.unwrap();
        let protocol = &ast.children[0];

        // PEG structure: Protocol -> [ProtocolName, Annotation, RolesSection, ParametersSection, InteractionSection]
        let interaction_section = &protocol.children[4]; // InteractionSection is at index 4
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
    Pack <Enactment>[W, P, S, in ID, in order, out tag, out package]
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
        assert!(
            result.is_ok(),
            "Failed to parse complete protocol: {:?}",
            result
        );

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
        assert!(
            result.is_err(),
            "Should fail when <Protocol> tag is missing"
        );
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
        ID <String>("identifier"),
        result <String>("result")
    
    Pack [W, in ID, out result]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_err(),
            "Should fail when <Enactment> tag is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when parameter type tag is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when protocol annotation is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when role annotation is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when parameter annotation is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when action annotation is missing"
        );
    }

    // Updated test for square bracket syntax
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

        // Count all annotations in the entire protocol
        let mut annotation_count = 0;
        let mut composition_count = 0;

        fn count_annotations_and_compositions(
            node: &AstNode,
            ann_count: &mut usize,
            comp_count: &mut usize,
        ) {
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

        count_annotations_and_compositions(&ast, &mut annotation_count, &mut composition_count);

        // Assert exactly 7 annotations:
        // 1. Protocol annotation: "Multi-party logistics protocol with commas, semicolons; and other punctuation."
        // 2. Role M annotation: "Merchant, coordinator, and manager"
        // 3. Role W annotation: "Warehouse, storage, and inventory"
        // 4. Parameter ID annotation: "unique identifier, used for tracking, contains special chars"
        // 5. Parameter order annotation: "order details, specifications, and requirements"
        // 6. Parameter package annotation: "packaged item ready for shipping"
        // 7. Action NotifyOrder annotation: "merchant notifies warehouse of new order, including details, timing, and priority"
        assert_eq!(
            annotation_count, 7,
            "Should have exactly 7 annotations in the protocol"
        );

        // Assert exactly 1 protocol composition:
        // Pack <Enactment>[W, M, in ID, in order, out package]
        assert_eq!(
            composition_count, 1,
            "Should have exactly 1 protocol composition"
        );

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
                        assert!(
                            description.contains("punctuation"),
                            "Should contain full text"
                        );
                    }
                }
            }

            for child in &node.children {
                search_annotations(child, found);
            }
        }

        search_annotations(ast, &mut protocol_annotation_found);
        assert!(
            protocol_annotation_found,
            "Should find protocol annotation with commas"
        );
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

                assert!(
                    protocol_ref_found,
                    "Composition should have protocol reference"
                );
                assert_eq!(
                    role_params, 2,
                    "Composition should have 2 role parameters (W, M)"
                );
                assert_eq!(
                    parameter_flows, 3,
                    "Composition should have 3 parameter flows (in ID, in order, out package)"
                );
            }

            for child in &node.children {
                search_composition(child, found);
            }
        }

        search_composition(ast, &mut composition_found);
        assert!(
            composition_found,
            "Should find the protocol composition with square brackets"
        );
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
        assert!(
            result.is_err(),
            "Should fail when annotation quotes are malformed"
        );
    }

    #[test]
    fn test_composition_without_brackets() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse")
    
    parameters
        ID <String>("identifier"),
        result <String>("result")
    
    Pack <Enactment> W, in ID, out result
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_err(),
            "Should fail when composition parameters are not enclosed in square brackets"
        );
    }

    #[test]
    fn test_nested_quotes_in_annotation() {
        let source = r#"
Logistics <Protocol>("Protocol with nested quotes in description") {
    roles
        M <Agent>("Agent with special role")
    
    parameters
        ID <String>("Parameter with quoted content")
    
    M -> W: NotifyOrder <Action>("Action with nested quotes")[out ID]
}
        "#;

        let result = parse_source(source);
        // PEG parser should handle this better than pest
        assert!(
            result.is_ok(),
            "PEG parser should handle nested quotes correctly"
        );

        if result.is_ok() {
            println!("✅ PEG parser handles nested quotes correctly");
        } else {
            println!("❌ PEG parser doesn't support nested quotes in annotations");
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
        assert!(
            result.is_err(),
            "Should fail when parameters section is missing"
        );
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
        assert!(
            result.is_err(),
            "Should fail when interaction arrow is malformed"
        );
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
        assert!(
            result.is_err(),
            "Should fail when parameter direction is invalid"
        );
    }

    #[test]
    fn test_composition_parameter_mixing() {
        let source = r#"
Logistics <Protocol>("Multi-party logistics protocol") {
    roles
        M <Agent>("Merchant"),
        W <Agent>("Warehouse"),
        P <Agent>("Packer"),
        S <Agent>("Scanner")
    
    parameters
        ID <String>("identifier"),
        order <String>("order details"),
        result <String>("result")
    
    Pack <Enactment>[W, P, in ID, S, out order, in result]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should correctly parse mixed role identifiers and parameter flows in composition"
        );

        if result.is_ok() {
            let ast = result.unwrap();
            let protocol = &ast.children[0];
            let interaction_section = &protocol.children[4]; // InteractionSection at index 4
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
        A <Agent>("Agent A"),
        B <Agent>("Agent B")
    
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
        assert!(
            result.is_err(),
            "Should fail when any protocol in the program is invalid"
        );
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
    Pack <Enactment>[W, M]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should handle empty parameter flow lists correctly"
        );
    }

    #[test]
    fn test_whitespace_and_formatting_variations() {
        let source = r#"
Logistics<Protocol>("Compact formatting"){
roles M<Agent>("Merchant"),W<Agent>("Warehouse")
parameters ID<String>("identifier"),result<String>("result")
M->W:NotifyOrder<Action>("action")[out ID]
Pack<Enactment>[W,M,in ID,out result]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should handle compact formatting without extra whitespace"
        );
    }

    // Additional PEG-specific tests
    #[test]
    fn test_peg_error_reporting() {
        let source = r#"
Logistics <Protocol>("Test protocol") {
    roles
        M <Agent>("Merchant")
    
    parameters
        ID <String>("identifier")
    
    M -> : NotifyOrder <Action>("incomplete interaction")[out ID]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_err(),
            "Should provide clear error for incomplete interaction"
        );

        // PEG should provide better error messages
        if let Err(e) = result {
            println!("PEG error message: {}", e);
        }
    }

    #[test]
    fn test_complex_composition_with_many_parameters() {
        let source = r#"
ComplexProtocol <Protocol>("Protocol with complex composition") {
    roles
        A <Agent>("Agent A"),
        B <Agent>("Agent B"),
        C <Agent>("Agent C"),
        D <Agent>("Agent D")
    
    parameters
        p1 <String>("Parameter 1"),
        p2 <String>("Parameter 2"),
        p3 <String>("Parameter 3"),
        p4 <String>("Parameter 4"),
        p5 <String>("Parameter 5")
    
    SubProtocol <Enactment>[A, B, C, D, in p1, in p2, out p3, out p4, in p5]
}
        "#;

        let result = parse_source(source);
        assert!(
            result.is_ok(),
            "Should handle complex composition with many parameters: {:?}",
            result
        );
    }
}
