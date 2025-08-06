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
        assert!(error_msg.contains("action1") || error_msg.contains("action2"),
               "Error should contain actual action names, got: {}", error_msg);
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
InvalidDirectionProtocol ("protocol with invalid direction") {
    roles
        A ("agent A"),
        B ("agent B")
    
    parameters
        param1 ("test parameter")
    
    A -> B: action1 ("action with invalid direction")[invalid param1]
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
CausalityChainProtocol <Protocol>("protocol with proper causality chain") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B"),
        C <Agent>("agent C")
    
    parameters
        input <String>("initial input"),
        intermediate <String>("intermediate value"),
        output <String>("final output")
    
    A -> B: step1 <Action>("first step - A provides input to B")[out input]
    B -> C: step2 <Action>("second step - B processes input and provides intermediate to C")[in input, out intermediate]
    C -> A: step3 <Action>("final step - C processes intermediate and provides output to A")[in intermediate, out output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        assert!(result.is_ok(), "Valid causality chain should pass validation: {:?}", result);
        
        if let Ok(_) = result {
            println!("✅ Causality chain validation passed");
            println!("Execution order: step1 (A->B) → step2 (B->C) → step3 (C->A)");
            println!("Parameter flow: input → intermediate → output");
        }
        
        Ok(())
    }

    #[test]
    fn test_causality_violation_detection() -> Result<()> {
        let bmpp_source = r#"
    CausalityViolationProtocol <Protocol>("protocol with causality violation") {
        roles
            A <Agent>("agent A"),
            B <Agent>("agent B")
        
        parameters
            param1 <String>("parameter 1"),
            param2 <String>("parameter 2")
        
        A -> B: action1 <Action>("action that needs param2")[in param2, out param1]
        B -> A: action2 <Action>("action that needs param1")[in param1, out param2]
    }
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        assert!(result.is_err(), "Circular dependency should be detected as causality violation");
        
        if let Err(e) = result {
            println!("✅ Causality violation correctly detected: {}", e);
            assert!(e.to_string().contains("causality") || e.to_string().contains("Circular dependency"));
            assert!(e.to_string().contains("action1") || e.to_string().contains("action2"),
                   "Error should contain actual action names, got: {}", e);
        }
        
        Ok(())
    }

    // distinguish between true multiple producers (safety violation) and parallel producers (valid BSPL pattern).
    #[test]
    fn test_valid_parallel_causality() -> Result<()> {
        let bmpp_source = r#"
ParallelProtocol <Protocol>("protocol with parallel causality") {
    roles
        Initiator <Agent>("starts the process"),
        ProcessorA <Agent>("processes branch A"),
        ProcessorB <Agent>("processes branch B"),
        Collector <Agent>("collects results")
    
    parameters
        input <String>("initial input"),
        resultA <String>("result from branch A"),
        resultB <String>("result from branch B"),
        final_output <String>("combined final output")
    
    Initiator -> ProcessorA: startA <Action>("start branch A")[out input]
    Initiator -> ProcessorB: startB <Action>("start branch B")[out input]
    ProcessorA -> Collector: finishA <Action>("complete branch A")[in input, out resultA]
    ProcessorB -> Collector: finishB <Action>("complete branch B")[in input, out resultB]
    Collector -> Initiator: combine <Action>("combine results")[in resultA, in resultB, out final_output]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        assert!(result.is_ok(), "Valid parallel causality should pass validation: {:?}", result);
        Ok(())
    }

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
        
        // The main protocol validation should fail because 'result' is never consumed
        let result = validate_parameter_flow(&ast);
        assert!(result.is_err(), "Protocol with unused output parameter should fail validation: {:?}", result);
        
        // But composition validation should pass
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
        
        // This should fail validation because 'orphan' is never produced and is not a pre-protocol parameter
        assert!(result.is_err(), "Protocol with unreachable interaction should not be valid: {:?}", result);
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("consumed but never produced") || error_msg.contains("completeness"),
               "Error should mention missing producer, got: {}", error_msg);
        
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
        
        // Should fail validation because 'input' is consumed but never produced (and is not a pre-protocol parameter)
        assert!(result.is_err(), "Incomplete protocol should not pass basic validation: {:?}", result);
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("consumed but never produced") || error_msg.contains("completeness"),
               "Error should mention completeness violation, got: {}", error_msg);
        
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

    #[test]
    fn test_action_name_resolution_in_errors() {
        let bmpp_source = r#"
ActionNameTestProtocol <Protocol>("protocol to test action name resolution in errors") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B"),
        C <Agent>("agent C")
    
    parameters
        param1 <String>("parameter 1"),
        param2 <String>("parameter 2"),
        param3 <String>("parameter 3")
    
    A -> B: step_one <Action>("first step")[in param3, out param1]
    B -> C: step_two <Action>("second step")[in param1, out param2]
    C -> A: step_three <Action>("third step")[in param2, out param3]
}
        "#;

        let ast = parse_source(bmpp_source).unwrap();
        let result = validate_parameter_flow(&ast);
        assert!(result.is_err(), "Circular dependency should be detected");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(!error_msg.contains("unknown_action"), 
               "Error should not contain 'unknown_action', got: {}", error_msg);
        assert!(error_msg.contains("step_one") || error_msg.contains("step_two") || error_msg.contains("step_three"),
               "Error should contain actual action names, got: {}", error_msg);
    }

    #[test]
    fn test_valid_linear_protocol_with_multiple_parameters() -> Result<()> {
        let bmpp_source = r#"
LinearMultiParamProtocol <Protocol>("linear protocol with multiple parameters per interaction") {
    roles
        Client <Agent>("client role"),
        Server <Agent>("server role"),
        Database <Agent>("database role")
    
    parameters
        user_id <String>("user identifier"),
        query <String>("database query"),
        auth_token <String>("authentication token"),
        results <String>("query results"),
        formatted_response <String>("formatted response")
    
    Client -> Server: authenticate <Action>("authenticate user")[out user_id, out auth_token]
    Server -> Database: query_db <Action>("query database")[in user_id, in auth_token, out query, out results]
    Database -> Client: respond <Action>("send formatted response")[in query, in results, out formatted_response]
}
        "#;

        let ast = parse_source(bmpp_source)?;
        let result = validate_parameter_flow(&ast);
        assert!(result.is_ok(), "Valid linear protocol with multiple parameters should pass: {:?}", result);
        Ok(())
    }
}
