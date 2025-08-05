#[cfg(test)]
mod tests {
    use crate::transpiler::parser::parse_source;
    use crate::transpiler::validation::validate_parameter_flow;
    use anyhow::Result;

    #[test]
    fn test_pre_protocol_knowledge_emission() -> Result<()> {
        // Test that pre-protocol inputs are correctly emitted via 'out' parameters
        let bmpp_source = r#"
Purchase <Protocol>("test pre-protocol knowledge emission") {
    roles
        Buyer <Agent>("the party wanting to buy an item"),
        Seller <Agent>("the party selling the item")
    
    parameters
        item_id <String>("unique identifier for the item"),
        item_name <String>("the name or description of the product"),
        price <Float>("the cost of the item quoted by the seller")
    
    Buyer -> Seller: request_quote <Action>("request for a price quote")[out item_id, out item_name]
    Seller -> Buyer: provide_quote <Action>("provide a price quote for requested item")[in item_id, in item_name, out price]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation - buyer emits pre-protocol knowledge
        assert!(result.is_ok(), "Pre-protocol knowledge emission should be valid");
        
        Ok(())
    }

    #[test]
    fn test_circular_dependency_detection() {
        // Test that circular dependencies are detected and prevented
        let bmpp_source = r#"
BadProtocol <Protocol>("protocol with circular dependency") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        param1 <String>("parameter 1"),
        param2 <String>("parameter 2")
    
    A -> B: action1 <Action>("first action")[in param2, out param1]
    B -> A: action2 <Action>("second action")[in param1, out param2]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_parameter_flow(&ast);
        
        // Should fail validation due to circular dependency
        assert!(result.is_err(), "Circular dependency should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Circular dependency detected"), 
               "Error should mention circular dependency");
    }

    #[test]
    fn test_multiple_producers_violation() {
        // Test that multiple producers for the same parameter are detected
        let bmpp_source = r#"
MultiProducerProtocol <Protocol>("protocol with multiple producers") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B"),
        C <Agent>("agent C")
    
    parameters
        shared_param <String>("parameter with multiple producers")
    
    A -> B: action1 <Action>("first action")[out shared_param]
    C -> B: action2 <Action>("second action")[out shared_param]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_parameter_flow(&ast);
        
        // Should fail validation due to multiple producers
        assert!(result.is_err(), "Multiple producers should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("multiple interactions"), 
               "Error should mention multiple producers");
    }

    #[test]
    fn test_unconsumed_parameters_warning() -> Result<()> {
        // Test that unused parameters generate warnings (but don't fail validation)
        let bmpp_source = r#"
UnusedParamProtocol <Protocol>("protocol with unused parameters") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        used_param <String>("parameter that is used"),
        unused_param <String>("parameter that is never used")
    
    A -> B: action1 <Action>("only action")[out used_param]
    B -> A: action2 <Action>("response action")[in used_param]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation (warnings don't cause failure)
        assert!(result.is_ok(), "Unused parameters should not fail validation");
        
        Ok(())
    }

    #[test]
    fn test_consumer_without_producer() {
        // Test that parameters consumed without producers are detected
        let bmpp_source = r#"
NoProducerProtocol <Protocol>("protocol with consumer but no producer") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        orphan_param <String>("parameter consumed but never produced")
    
    A -> B: action1 <Action>("consumes orphan param")[in orphan_param]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_parameter_flow(&ast);
        
        // Should fail validation
        assert!(result.is_err(), "Consumer without producer should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("consumed but never produced"), 
               "Error should mention missing producer");
    }

    #[test]
    fn test_special_id_parameter_handling() -> Result<()> {
        // Test that ID parameter is handled specially (can be consumed without explicit producer)
        let bmpp_source = r#"
IDProtocol <Protocol>("protocol with special ID parameter handling") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        ID <String>("special identifier parameter")
    
    A -> B: action1 <Action>("uses ID without explicit producer")[in ID]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation - ID is special
        assert!(result.is_ok(), "ID parameter should be handled specially");
        
        Ok(())
    }

    #[test]
    fn test_undeclared_parameter_usage() {
        // Test that using undeclared parameters is detected
        let bmpp_source = r#"
UndeclaredParamProtocol <Protocol>("protocol using undeclared parameter") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        declared_param <String>("parameter that is declared")
    
    A -> B: action1 <Action>("uses undeclared param")[out undeclared_param]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_parameter_flow(&ast);
        
        // Should fail validation
        assert!(result.is_err(), "Undeclared parameter usage should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("is not declared"), 
               "Error should mention undeclared parameter");
    }

    #[test]
    fn test_valid_complex_protocol() -> Result<()> {
        // Test a complex but valid protocol that follows all BSPL principles
        let bmpp_source = r#"
ComplexValidProtocol <Protocol>("complex but valid protocol") {
    roles
        Buyer <Agent>("the buyer"),
        Seller <Agent>("the seller"),
        Shipper <Agent>("the shipper")
    
    parameters
        ID <String>("request identifier"),
        item <String>("item description"),
        price <Float>("quoted price"),
        address <String>("shipping address"),
        shipped <Bool>("shipment status"),
        delivered <Bool>("delivery confirmation")
    
    Buyer -> Seller: rfq <Action>("request for quote")[out ID, out item]
    Seller -> Buyer: quote <Action>("provide quote")[in ID, in item, out price]
    Buyer -> Seller: accept <Action>("accept quote")[in ID, in price, out address]
    Seller -> Shipper: ship <Action>("ship item")[in ID, in item, in address, out shipped]
    Shipper -> Buyer: deliver <Action>("confirm delivery")[in ID, in shipped, out delivered]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass all validations
        assert!(result.is_ok(), "Complex valid protocol should pass validation");
        
        Ok(())
    }

    #[test]
    fn test_parameter_direction_validation() {
        // Test that invalid parameter directions are caught
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

        // This should fail at the parser level, not validation level
        let result = parse_source(bmpp_source);
        assert!(result.is_err(), "Invalid parameter direction should be caught by parser");
    }

    #[test]
    fn test_empty_parameter_flow() -> Result<()> {
        // Test that empty parameter flows are handled correctly
        let bmpp_source = r#"
EmptyFlowProtocol <Protocol>("protocol with empty parameter flow") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        dummy <String>("dummy parameter to satisfy non-empty requirement")
    
    A -> B: action1 <Action>("action with no parameters")[]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation
        assert!(result.is_ok(), "Empty parameter flow should be valid");
        
        Ok(())
    }

    #[test]
    fn test_causality_chain_validation() -> Result<()> {
        // Test that proper causality chains are validated correctly
        let bmpp_source = r#"
CausalityChainProtocol <Protocol>("protocol with causality chain") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B"),
        C <Agent>("agent C")
    
    parameters
        input <String>("initial input"),
        intermediate <String>("intermediate value"),
        output <String>("final output")
    
    A -> B: step1 <Action>("first step")[out input]
    B -> C: step2 <Action>("second step")[in input, out intermediate]
    C -> A: step3 <Action>("final step")[in intermediate, out output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation - proper causality chain
        assert!(result.is_ok(), "Valid causality chain should pass validation");
        
        Ok(())
    }
}
