// tests/payload_tests.rs (or wherever these tests are located)
use anyhow::Result;
use bmpp_agents::transpiler::parser::parse_source;

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
    assert_eq!(protocol.get_string("name").unwrap(), "GreetingProtocol");
    assert_eq!(protocol.get_string("description").unwrap(), "a protocol for generating friendly greetings");
    
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
    assert_eq!(protocol.get_string("name").unwrap(), "JokeProtocol");
    assert_eq!(protocol.get_string("description").unwrap(), "a protocol for generating humorous content");
    
    // Verify parameters section contains the joke parameter
    let params_section = protocol.children.iter()
        .find(|child| child.node_type == bmpp_agents::protocol::ast::AstNodeType::ParametersSection)
        .expect("Parameters section should exist");
    
    let joke_param = params_section.children.iter()
        .find(|param| param.get_string("name").unwrap() == "joke")
        .expect("joke parameter should exist");
    
    assert_eq!(joke_param.get_string("type").unwrap(), "String");
    assert_eq!(joke_param.get_string("description").unwrap(), "a short humorous line");
    
    Ok(())
}
