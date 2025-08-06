// tests/payload_tests.rs (or wherever these tests are located)
use anyhow::Result;
use bmpp_agents::transpiler::parser::parse_source;
use bmpp_agents::protocol::ast::AstNodeType;

#[test]
fn test_greeting_generation_payload_with_inline_meaning() -> Result<()> {
    let bmpp_source = r#"
GreetingProtocol <Protocol>("a protocol for generating friendly greetings") {
    roles
        Requester <Agent>("the party requesting a greeting"),
        Responder <Agent>("the party providing the greeting")
    
    parameters
        greeting <String>("a friendly greeting message"),
        context <String>("contextual information for the greeting")
    
    Requester -> Responder: request_greeting <Action>("request a personalized greeting")[out context]
    Responder -> Requester: provide_greeting <Action>("provide the requested greeting")[in context, out greeting]
}
    "#;
    
    let ast = parse_source(bmpp_source)?;
    
    // Verify the protocol was parsed correctly
    assert_eq!(ast.children.len(), 1);
    let protocol = &ast.children[0];
    assert_eq!(protocol.node_type, AstNodeType::Protocol);
    
    // Use the convenience accessor methods
    assert_eq!(protocol.get_protocol_name().unwrap(), "GreetingProtocol");
    assert_eq!(protocol.get_protocol_annotation().unwrap(), "a protocol for generating friendly greetings");
    
    Ok(())
}

#[test]
fn test_joke_generation_payload() -> Result<()> {
    let bmpp_source = r#"
JokeProtocol <Protocol>("a protocol for generating humorous content") {
    roles
        Requester <Agent>("the party requesting a joke"),
        Comedian <Agent>("the party providing the joke")
    
    parameters
        joke <String>("a short humorous line"),
        topic <String>("the subject matter for the joke"),
        style <String>("the comedic style preference")
    
    Requester -> Comedian: request_joke <Action>("request a joke on a specific topic")[out topic, out style]
    Comedian -> Requester: deliver_joke <Action>("provide the requested joke")[in topic, in style, out joke]
}
    "#;
    
    let ast = parse_source(bmpp_source)?;
    
    // Verify the protocol was parsed correctly
    assert_eq!(ast.children.len(), 1);
    let protocol = &ast.children[0];
    assert_eq!(protocol.node_type, AstNodeType::Protocol);
    
    // Use the convenience accessor methods for protocol info
    assert_eq!(protocol.get_protocol_name().unwrap(), "JokeProtocol");
    assert_eq!(protocol.get_protocol_annotation().unwrap(), "a protocol for generating humorous content");
    
    // Verify parameters section contains the joke parameter using accessor methods
    let params_section = protocol.get_parameters_section()
        .expect("Parameters section should exist");
    
    let param_declarations = params_section.get_parameter_declarations();
    let joke_param = param_declarations.iter()
        .find(|param| {
            if let Some((name, _, _)) = param.get_parameter_decl_info() {
                name == "joke"
            } else {
                false
            }
        })
        .expect("joke parameter should exist");
    
    // Use the convenience method to get parameter declaration info
    let (name, param_type, description) = joke_param.get_parameter_decl_info().unwrap();
    assert_eq!(name, "joke");
    assert_eq!(param_type, "String");
    assert_eq!(description, "a short humorous line");
    
    Ok(())
}
