use anyhow::Result;
use bmpp_agents::transpiler::{parser::parse_source, validation::validate_parameter_flow};

#[test]
fn test_bspl_principle_1_pre_protocol_knowledge() -> Result<()> {
    // BSPL Principle 1: Pre-protocol knowledge must be emitted via 'out' parameters
    let protocol = r#"
ProtocolKnowledge <Protocol>("test pre-protocol knowledge principle") {
    roles
        Customer <Agent>("initiating party with pre-protocol knowledge"),
        Service <Agent>("responding party")
    
    parameters
        customer_id <String>("customer's pre-existing identifier"),
        service_request <String>("specific service being requested"),
        quote <Float>("service quote provided by service")
    
    Customer -> Service: request_service <Action>("customer initiates with known info")[out customer_id, out service_request]
    Service -> Customer: provide_quote <Action>("service responds with quote")[in customer_id, in service_request, out quote]
}
    "#;

    let ast = parse_source(protocol)?;
    let result = validate_parameter_flow(&ast);

    assert!(
        result.is_ok(),
        "Customer should be able to emit pre-protocol knowledge (customer_id, service_request)"
    );

    Ok(())
}

#[test]
fn test_bspl_principle_2_no_multiple_producers() {
    // BSPL Principle 2: Each parameter should have at most one producer
    let protocol = r#"
MultipleProducers <Protocol>("test multiple producers violation") {
    roles
        A <Agent>("first producer"),
        B <Agent>("second producer"),
        C <Agent>("consumer")
    
    parameters
        conflicted_param <String>("parameter with multiple producers")
    
    A -> C: produce_a <Action>("first producer action")[out conflicted_param]
    B -> C: produce_b <Action>("second producer action")[out conflicted_param]
}
    "#;

    let ast = parse_source(protocol).unwrap();
    let result = validate_parameter_flow(&ast);

    assert!(
        result.is_err(),
        "Multiple producers should be detected as a safety violation"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("multiple interactions"),
        "Error should specifically mention multiple producers"
    );
}

#[test]
fn test_bspl_principle_3_causality_preservation() {
    // BSPL Principle 3: Parameter dependencies must not create cycles
    let protocol = r#"
CyclicDependency <Protocol>("test cyclic dependency detection") {
    roles
        A <Agent>("agent A"),
        B <Agent>("agent B")
    
    parameters
        param_x <String>("parameter X"),
        param_y <String>("parameter Y")
    
    A -> B: action1 <Action>("A depends on Y, produces X")[in param_y, out param_x]
    B -> A: action2 <Action>("B depends on X, produces Y")[in param_x, out param_y]
}
    "#;

    let ast = parse_source(protocol).unwrap();
    let result = validate_parameter_flow(&ast);

    assert!(result.is_err(), "Circular dependency should be detected");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"),
        "Error should specifically mention circular dependency"
    );
}

#[test]
fn test_bspl_principle_4_complete_information_flow() {
    // BSPL Principle 4: All consumed parameters must have producers
    let protocol = r#"
IncompleteFlow <Protocol>("test incomplete information flow") {
    roles
        Consumer <Agent>("consumes without producer"),
        Other <Agent>("other party")
    
    parameters
        orphaned_param <String>("parameter consumed but never produced")
    
    Consumer -> Other: consume_orphan <Action>("consumes parameter without producer")[in orphaned_param]
}
    "#;

    let ast = parse_source(protocol).unwrap();
    let result = validate_parameter_flow(&ast);

    assert!(
        result.is_err(),
        "Orphaned parameter consumption should be detected"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("consumed but never produced"),
        "Error should mention missing producer"
    );
}

#[test]
fn test_valid_purchase_protocol_follows_bspl() -> Result<()> {
    // Test that a properly designed purchase protocol follows all BSPL principles
    let protocol = r#"
ValidPurchase <Protocol>("BSPL-compliant purchase protocol") {
    roles
        Buyer <Agent>("party making purchase"),
        Seller <Agent>("party providing goods"),
        Shipper <Agent>("party handling logistics")
    
    parameters
        request_id <String>("buyer's request identifier"),
        item <String>("item buyer wants to purchase"),
        price <Float>("seller's quoted price"),
        payment <Bool>("payment confirmation"),
        address <String>("shipping destination"),
        shipped <Bool>("shipment confirmation"),
        delivered <Bool>("delivery confirmation")
    
    Buyer -> Seller: request_quote <Action>("buyer requests quote with pre-protocol knowledge")[out request_id, out item]
    Seller -> Buyer: provide_quote <Action>("seller quotes price")[in request_id, in item, out price]
    Buyer -> Seller: confirm_purchase <Action>("buyer confirms purchase")[in request_id, in price, out payment, out address]
    Seller -> Shipper: arrange_shipping <Action>("seller arranges shipping")[in request_id, in item, in address, out shipped]
    Shipper -> Buyer: confirm_delivery <Action>("shipper confirms delivery")[in request_id, in shipped, out delivered]
}
    "#;

    let ast = parse_source(protocol)?;
    let result = validate_parameter_flow(&ast);

    assert!(
        result.is_ok(),
        "Valid purchase protocol should pass all BSPL validations"
    );

    Ok(())
}

#[test]
fn test_parameter_flow_directions() {
    let source = r#"
DirectionsTest <Protocol>("test parameter flow directions") {
    roles
        Producer <Agent>("produces parameters"),
        Consumer <Agent>("consumes parameters")
    
    parameters
        produced_param <String>("parameter that gets produced"),
        consumed_param <String>("parameter that gets consumed")
    
    Producer -> Consumer: transfer <Action>("transfer with proper directions")[out produced_param, in consumed_param]
}
    "#;

    use bmpp_agents::protocol::ast::AstNodeType;

    let result = parse_source(source);
    assert!(result.is_ok(), "Should parse successfully: {:?}", result);

    let ast = result.unwrap();
    println!("DEBUG: AST {}", ast);

    // Get the protocol using accessor methods
    let protocol_node = &ast.children[0];
    assert_eq!(protocol_node.node_type, AstNodeType::Protocol);

    // Get the interactions section safely
    let interactions_section = protocol_node
        .get_interactions_section()
        .expect("Protocol should have an interactions section");
    println!("DEBUG: interactions_section {}", interactions_section);

    // Get the interaction items
    let interaction_items = interactions_section.get_interaction_items();
    assert_eq!(
        interaction_items.len(),
        1,
        "Should have exactly one interaction"
    );

    let interaction_item = interaction_items[0];
    println!("DEBUG: interaction_item {}", interaction_item);

    // Get the standard interaction
    let standard_interaction = interaction_item
        .get_standard_interaction()
        .expect("Interaction item should contain a standard interaction");
    println!("DEBUG: standard_interaction {}", standard_interaction);

    // Get interaction information using the helper method
    let interaction_info = standard_interaction
        .get_standard_interaction_info()
        .expect("Should be able to extract interaction info");

    assert_eq!(interaction_info.from_role, "Producer");
    assert_eq!(interaction_info.to_role, "Consumer");
    assert_eq!(interaction_info.action_name, "transfer");
    assert_eq!(interaction_info.parameter_flows.len(), 2);

    // Verify parameter flow directions
    let out_flows: Vec<_> = interaction_info
        .parameter_flows
        .iter()
        .filter(|(direction, _)| direction == "out")
        .collect();
    let in_flows: Vec<_> = interaction_info
        .parameter_flows
        .iter()
        .filter(|(direction, _)| direction == "in")
        .collect();

    assert_eq!(out_flows.len(), 1, "Should have exactly one out parameter");
    assert_eq!(in_flows.len(), 1, "Should have exactly one in parameter");

    assert_eq!(out_flows[0].1, "produced_param");
    assert_eq!(in_flows[0].1, "consumed_param");

    println!("✅ Parameter flow directions are correctly enforced");
}
