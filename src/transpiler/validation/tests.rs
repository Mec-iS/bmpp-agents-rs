#[cfg(test)]
mod tests {
    use crate::transpiler::parser::parse_source;
    use crate::transpiler::validation::{validate_parameter_flow, validate_protocol_composition};
    use anyhow::Result;

    #[test]
    fn test_pre_protocol_knowledge_emission() -> Result<()> {
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
        
        assert!(result.is_ok(), "Pre-protocol knowledge emission should be valid: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_circular_dependency_detection() {
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
        
        assert!(result.is_err(), "Circular dependency should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Circular dependency detected"), 
               "Error should mention circular dependency, got: {}", error_msg);
    }

    #[test]
    fn test_multiple_producers_violation() {
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
        
        assert!(result.is_err(), "Multiple producers should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("multiple interactions") || error_msg.contains("BSPL safety violation"), 
               "Error should mention multiple producers or BSPL violation, got: {}", error_msg);
    }

    #[test]
    fn test_unconsumed_parameters_warning() -> Result<()> {
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
        
        assert!(result.is_ok(), "Unused parameters should not fail validation: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_consumer_without_producer() {
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
        
        assert!(result.is_err(), "Consumer without producer should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("consumed but never produced") || error_msg.contains("BSPL completeness violation"), 
               "Error should mention missing producer or BSPL violation, got: {}", error_msg);
    }

    #[test]
    fn test_special_id_parameter_handling() -> Result<()> {
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
        
        assert!(result.is_ok(), "ID parameter should be handled specially: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_undeclared_parameter_usage() {
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
        
        assert!(result.is_err(), "Undeclared parameter usage should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("is not declared"), 
               "Error should mention undeclared parameter, got: {}", error_msg);
    }

    #[test]
    fn test_valid_complex_protocol() -> Result<()> {
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
        
        assert!(result.is_ok(), "Complex valid protocol should pass validation: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_parameter_direction_validation() {
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
        assert!(result.is_err(), "Invalid parameter direction should be caught by parser");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("expected") || error_msg.contains("Direction"), 
            "Error should mention parsing issue or Direction, got: {}", error_msg);
    }

    #[test]
    fn test_empty_parameter_flow() -> Result<()> {
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
        
        assert!(result.is_ok(), "Empty parameter flow should be valid: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_causality_chain_validation() -> Result<()> {
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
        
        assert!(result.is_ok(), "Valid causality chain should pass validation: {:?}", result);
        
        Ok(())
    }

    // New tests for enhanced BSPL validation

    #[test]
    fn test_protocol_composition_validation() -> Result<()> {
        let bmpp_source = r#"
MainProtocol <Protocol>("protocol with composition") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        id <String>("identifier"),
        result <String>("result")
    
    A -> B: start <Action>("start action")[out id]
    SubProtocol <Enactment>[A, B, in id, out result]
}

SubProtocol <Protocol>("sub protocol") {
    roles
        X <Agent>("agent X"),
        Y <Agent>("agent Y")
    
    parameters
        input <String>("input parameter"),
        output <String>("output parameter")
    
    X -> Y: process <Action>("process data")[in input, out output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        assert!(result.is_err(), "Never consumed parameter should not pass: {:?}", result);
        
        let comp_result = validate_protocol_composition(&ast);
        assert!(comp_result.is_ok(), "Protocol composition validation should pass: {:?}", comp_result);
        
        Ok(())
    }

    #[test]
    fn test_invalid_protocol_reference() {
        let bmpp_source = r#"
MainProtocol <Protocol>("protocol with invalid reference") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        id <String>("identifier")
    
    NonExistentProtocol <Enactment>[A, B, in id]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_protocol_composition(&ast);
        
        assert!(result.is_err(), "Invalid protocol reference should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("unknown protocol"), 
               "Error should mention unknown protocol, got: {}", error_msg);
    }

    #[test]
    fn test_self_referencing_protocol() {
        let bmpp_source = r#"
RecursiveProtocol <Protocol>("self-referencing protocol") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        counter <Int>("recursion counter")
    
    A -> B: check <Action>("check condition")[in counter]
    RecursiveProtocol <Enactment>[A, B, in counter]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_protocol_composition(&ast);
        
        assert!(result.is_err(), "Self-referencing protocol should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("cannot reference itself"), 
               "Error should mention self-reference, got: {}", error_msg);
    }

    #[test]
    fn test_enactability_validation() -> Result<()> {
        let bmpp_source = r#"
EnactableProtocol <Protocol>("protocol testing enactability") {
    roles
        Producer <Agent>("produces data"),
        Consumer <Agent>("consumes data"),
        Processor <Agent>("processes data")
    
    parameters
        raw_data <String>("initial data"),
        processed_data <String>("processed data"),
        final_result <String>("final result")
    
    Producer -> Processor: provide_data <Action>("provide raw data")[out raw_data]
    Processor -> Consumer: process <Action>("process the data")[in raw_data, out processed_data]
    Consumer -> Producer: finalize <Action>("finalize with result")[in processed_data, out final_result]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        assert!(result.is_ok(), "Enactable protocol should pass validation: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_unreachable_interaction() -> Result<()> {
        let bmpp_source = r#"
UnreachableProtocol <Protocol>("protocol with potentially unreachable interaction") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        trigger <String>("trigger parameter"),
        result <String>("result parameter"),
        orphan <String>("orphan parameter")
    
    A -> B: start <Action>("accessible start")[out trigger]
    B -> A: respond <Action>("accessible response")[in trigger, out result]
    A -> B: unreachable <Action>("unreachable action")[in orphan, out result]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // This should pass validation but may generate warnings about unreachable interactions
        assert!(result.is_err(), "Protocol with unreachable interaction should not be valid: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_protocol_completeness() -> Result<()> {
        let bmpp_source = r#"
IncompleteProtocol <Protocol>("protocol with completeness issues") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        input <String>("input parameter"),
        unused_output <String>("parameter that's produced but never consumed"),
        completely_unused <String>("parameter that's never used")
    
    A -> B: produce <Action>("produces output that's never used")[in input, out unused_output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        // Should pass validation but generate completeness warnings
        assert!(result.is_err(), "Incomplete protocol should not pass basic validation: {:?}", result);
        
        Ok(())
    }

    #[test]
    fn test_complex_causality_validation() -> Result<()> {
        let bmpp_source = r#"
ComplexCausalityProtocol <Protocol>("protocol with complex causality") {
    roles
        A <Agent>("initiator"),
        B <Agent>("processor"),
        C <Agent>("validator"),
        D <Agent>("finalizer")
    
    parameters
        request <String>("initial request"),
        data1 <String>("first processing result"),
        data2 <String>("second processing result"),
        validation <Bool>("validation result"),
        final_output <String>("final output")
    
    A -> B: initiate <Action>("start process")[out request]
    B -> C: process1 <Action>("first processing")[in request, out data1]
    B -> D: process2 <Action>("second processing")[in request, out data2]
    C -> D: validate <Action>("validate results")[in data1, out validation]
    D -> A: finalize <Action>("produce final output")[in data2, in validation, out final_output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        
        assert!(result.is_ok(), "Complex causality should be handled correctly: {:?}", result);
        
        Ok(())
    }
}
