pub mod tests;

use crate::protocol::ast::{AstNode, AstNodeType};
use anyhow::{anyhow, Result};
use peg::parser;

// PEG grammar embedded in Rust
peg::parser! {
    grammar bmpp_parser() for str {
        // Whitespace and comment handling
        rule _() = quiet!{[' ' | '\t' | '\n' | '\r']*}
        rule __() = quiet!{[' ' | '\t' | '\n' | '\r']+}

        // Comments: lines starting with //
        rule comment() = "//" [^'\n' | '\r']* ['\n' | '\r']?

        // Skip whitespace and comments
        rule ws() = (comment() / [' ' | '\t' | '\n' | '\r'])*

        // Basic tokens
        rule identifier() -> String
            = n:$(['a'..='z' | 'A'..='Z'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { n.to_string() }

        rule string_literal() -> String
            = "\"" s:$([^'"']*) "\"" { s.to_string() }

        // Semantic tags
        rule protocol_tag() = "<Protocol>"
        rule agent_tag() = "<Agent>"
        rule action_tag() = "<Action>"
        rule enactment_tag() = "<Enactment>"

        // Basic types
        rule basic_type() -> String
            = t:$("String" / "Int" / "Float" / "Bool") { t.to_string() }

        // Direction keywords
        rule direction() -> String
            = d:$("in" / "out") { d.to_string() }

        // Annotation: ("text with, commas and; semicolons")
        rule annotation() -> AstNode
            = "(" ws() s:string_literal() ws() ")" {
                let mut node = AstNode::new(AstNodeType::Annotation);
                node.set_string("description", &s);
                node
            }

        // Protocol name
        rule protocol_name() -> AstNode
            = name:identifier() {
                let mut node = AstNode::new(AstNodeType::ProtocolName);
                node.set_string("name", &name);
                node
            }

        // Role reference
        rule role_ref() -> AstNode
            = name:identifier() {
                let mut node = AstNode::new(AstNodeType::RoleRef);
                node.set_string("name", &name);
                node
            }

        // Action name
        rule action_name() -> AstNode
            = name:identifier() {
                let mut node = AstNode::new(AstNodeType::ActionName);
                node.set_string("name", &name);
                node
            }

        // Role declaration: RoleName <Agent>("description")
        rule role_decl() -> AstNode
            = name:identifier() ws() agent_tag() ws() ann:annotation() {
                let mut role_node = AstNode::new(AstNodeType::RoleDecl);

                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", &name);
                role_node.children.push(Box::new(identifier_node));

                role_node.children.push(Box::new(ann));
                role_node
            }

        // Roles section: roles Role1 <Agent>("desc"), Role2 <Agent>("desc")
        rule roles_section() -> AstNode
            = "roles" ws() roles:(role_decl() ** (ws() "," ws())) {
                let mut roles_node = AstNode::new(AstNodeType::RolesSection);
                for role in roles {
                    roles_node.children.push(Box::new(role));
                }
                roles_node
            }

        // Parameter declaration: paramName <Type>("description")
        rule parameter_decl() -> AstNode
            = name:identifier() ws() "<" ws() typ:basic_type() ws() ">" ws() ann:annotation() {
                let mut param_node = AstNode::new(AstNodeType::ParameterDecl);

                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", &name);
                param_node.children.push(Box::new(identifier_node));

                let mut type_node = AstNode::new(AstNodeType::BasicType);
                type_node.set_string("type", &typ);
                param_node.children.push(Box::new(type_node));

                param_node.children.push(Box::new(ann));
                param_node
            }

        // Parameters section
        rule parameters_section() -> AstNode
            = "parameters" ws() params:(parameter_decl() ** (ws() "," ws())) {
                let mut params_node = AstNode::new(AstNodeType::ParametersSection);
                for param in params {
                    params_node.children.push(Box::new(param));
                }
                params_node
            }

        // Parameter flow: in paramName or out paramName
        rule parameter_flow() -> AstNode
            = dir:direction() ws() name:identifier() {
                let mut flow_node = AstNode::new(AstNodeType::ParameterFlow);
                flow_node.set_string("direction", &dir);

                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", &name);
                flow_node.children.push(Box::new(identifier_node));

                flow_node
            }

        // Parameter flow list: [in param1, out param2]
        rule parameter_flow_list() -> Vec<AstNode>
            = "[" ws() flows:(parameter_flow() ** (ws() "," ws()))? ws() "]" {
                flows.unwrap_or_default()
            }

        // Standard interaction: RoleRef -> RoleRef : ActionName <Action> Annotation [ParameterFlowList]
        rule standard_interaction() -> AstNode
            = from:role_ref() ws() "->" ws() to:role_ref() ws() ":" ws() action:action_name() ws()
              action_tag() ws() ann:annotation() ws() flows:parameter_flow_list() {
                let mut interaction_node = AstNode::new(AstNodeType::StandardInteraction);

                // Add role references as children
                interaction_node.children.push(Box::new(from));
                interaction_node.children.push(Box::new(to));

                // Add action name
                interaction_node.children.push(Box::new(action));

                // Add annotation
                interaction_node.children.push(Box::new(ann));

                // Add parameter flows
                for flow in flows {
                    interaction_node.children.push(Box::new(flow));
                }

                interaction_node
            }

        // Protocol reference for composition
        rule protocol_reference() -> AstNode
            = name:identifier() ws() enactment_tag() {
                let mut protocol_ref_node = AstNode::new(AstNodeType::ProtocolReference);
                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", &name);
                protocol_ref_node.children.push(Box::new(identifier_node));
                protocol_ref_node
            }

        // Composition parameter: either just identifier or direction + identifier
        rule composition_parameter() -> AstNode
            = flow:parameter_flow() { flow }
            / name:identifier() {
                let mut identifier_node = AstNode::new(AstNodeType::Identifier);
                identifier_node.set_string("name", &name);
                identifier_node
            }

        // Composition parameter list
        rule composition_parameter_list() -> Vec<AstNode>
            = params:(composition_parameter() ** (ws() "," ws())) { params }

        // Protocol composition: ProtocolReference [CompositionParameterList]
        rule protocol_composition() -> AstNode
            = protocol_ref:protocol_reference() ws()
              "[" ws() params:composition_parameter_list()? ws() "]" {
                let mut composition_node = AstNode::new(AstNodeType::ProtocolComposition);

                // Add protocol reference
                composition_node.children.push(Box::new(protocol_ref));

                // Add composition parameters
                if let Some(params) = params {
                    for param in params {
                        composition_node.children.push(Box::new(param));
                    }
                }

                composition_node
            }

        // Interaction item: either standard interaction or protocol composition
        rule interaction_item() -> AstNode
            = item:(standard_interaction() / protocol_composition()) {
                let mut item_node = AstNode::new(AstNodeType::InteractionItem);
                item_node.children.push(Box::new(item));
                item_node
            }

        // Interactions section - contains multiple interaction items
        rule interactions_section() -> AstNode
            = items:interaction_item() ++ ws() {
                let mut interactions_node = AstNode::new(AstNodeType::InteractionSection);
                for item in items {
                    interactions_node.children.push(Box::new(item));
                }
                interactions_node
            }

        // Complete protocol: ProtocolName <Protocol> Annotation { RolesSection ParametersSection InteractionsSection }
        rule protocol() -> AstNode
            = name:protocol_name() ws() protocol_tag() ws() ann:annotation() ws() "{" ws()
              roles:roles_section() ws()
              params:parameters_section() ws()
              interactions:interactions_section() ws()
              "}" {
                let mut protocol_node = AstNode::new(AstNodeType::Protocol);
                protocol_node.children.push(Box::new(name));
                protocol_node.children.push(Box::new(ann));
                protocol_node.children.push(Box::new(roles));
                protocol_node.children.push(Box::new(params));
                protocol_node.children.push(Box::new(interactions));
                protocol_node
            }

        // Program: one or more protocols (with optional comments)
        pub rule program() -> AstNode
            = ws() protocols:protocol() ++ ws() ws() {
                let mut program_node = AstNode::new(AstNodeType::Program);
                for protocol in protocols {
                    program_node.children.push(Box::new(protocol));
                }
                program_node
            }
    }
}

// Updated parser function with better error handling
pub fn parse_source(source: &str) -> Result<AstNode> {
    match bmpp_parser::program(source) {
        Ok(ast) => Ok(ast),
        Err(e) => {
            // Enhanced error reporting
            Err(anyhow!("Parse error at position {}: {}", e.location, e))
        }
    }
}
