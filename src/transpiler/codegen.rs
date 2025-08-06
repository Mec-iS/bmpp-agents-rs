use crate::protocol::ast::{AstNode, AstNodeType};
use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Serialize)]
struct Protocol {
    name: String,
    description: String,
    roles: Vec<Role>,
    parameters: Vec<Parameter>,
    interactions: Vec<InteractionItem>,
}

#[derive(Serialize)]
struct Role {
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Parameter {
    name: String,
    param_type: String,
    description: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum InteractionItem {
    StandardInteraction(StandardInteraction),
    ProtocolComposition(ProtocolComposition),
}

#[derive(Serialize)]
struct StandardInteraction {
    from_role: String,
    to_role: String,
    action: String,
    description: String,
    parameter_flows: Vec<ParameterFlow>,
}

#[derive(Serialize)]
struct ProtocolComposition {
    protocol_name: String,
    roles: Vec<String>,
    parameter_flows: Vec<ParameterFlow>,
}

#[derive(Serialize)]
struct ParameterFlow {
    direction: String,
    parameter: String,
}

pub struct BmppCodeGenerator;

impl BmppCodeGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, ast: &AstNode) -> Result<String> {
        match ast.node_type {
            AstNodeType::Program => self.generate_program(ast),
            _ => Err(anyhow!("Expected Program node, got {:?}", ast.node_type)),
        }
    }

    fn generate_program(&self, ast: &AstNode) -> Result<String> {
        let mut protocols = Vec::new();

        // Process each protocol in the program
        for protocol_node in &ast.children {
            if protocol_node.node_type == AstNodeType::Protocol {
                let protocol = self.process_protocol(protocol_node)?;
                protocols.push(protocol);
            }
        }

        if protocols.is_empty() {
            return Err(anyhow!("No protocols found in AST"));
        }

        self.generate_rust_code(&protocols)
    }

    fn process_protocol(&self, node: &AstNode) -> Result<Protocol> {
        let mut name = "UnknownProtocol".to_string();
        let mut description = "No description".to_string();
        let mut roles = Vec::new();
        let mut parameters = Vec::new();
        let mut interactions = Vec::new();

        for child in &node.children {
            match child.node_type {
                AstNodeType::ProtocolName => {
                    if let Some(protocol_name) = child.get_string("name") {
                        name = protocol_name.clone();
                    }
                }
                AstNodeType::Annotation => {
                    if let Some(desc) = child.get_string("description") {
                        description = desc.clone();
                    }
                }
                AstNodeType::RolesSection => {
                    roles = self.process_roles(child)?;
                }
                AstNodeType::ParametersSection => {
                    parameters = self.process_parameters(child)?;
                }
                AstNodeType::InteractionSection => {
                    interactions = self.process_interaction_section(child)?;
                }
                _ => {}
            }
        }

        Ok(Protocol {
            name,
            description,
            roles,
            parameters,
            interactions,
        })
    }

    fn process_roles(&self, node: &AstNode) -> Result<Vec<Role>> {
        let mut roles = Vec::new();

        for child in &node.children {
            if child.node_type == AstNodeType::RoleDecl {
                let role = self.process_role_decl(child)?;
                roles.push(role);
            }
        }

        Ok(roles)
    }

    fn process_role_decl(&self, node: &AstNode) -> Result<Role> {
        let mut name = "UnknownRole".to_string();
        let mut description = "No description".to_string();

        for child in &node.children {
            match child.node_type {
                AstNodeType::Identifier => {
                    if let Some(role_name) = child.get_string("name") {
                        name = role_name.clone();
                    }
                }
                AstNodeType::Annotation => {
                    if let Some(desc) = child.get_string("description") {
                        description = desc.clone();
                    }
                }
                _ => {}
            }
        }

        Ok(Role { name, description })
    }

    fn process_parameters(&self, node: &AstNode) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();

        for child in &node.children {
            if child.node_type == AstNodeType::ParameterDecl {
                let parameter = self.process_parameter_decl(child)?;
                parameters.push(parameter);
            }
        }

        Ok(parameters)
    }

    fn process_parameter_decl(&self, node: &AstNode) -> Result<Parameter> {
        let mut name = "unknown_param".to_string();
        let mut param_type = "String".to_string();
        let mut description = "No description".to_string();

        for child in &node.children {
            match child.node_type {
                AstNodeType::Identifier => {
                    if let Some(param_name) = child.get_string("name") {
                        name = param_name.clone();
                    }
                }
                AstNodeType::BasicType => {
                    if let Some(type_name) = child.get_string("type") {
                        param_type = type_name.clone();
                    }
                }
                AstNodeType::Annotation => {
                    if let Some(desc) = child.get_string("description") {
                        description = desc.clone();
                    }
                }
                _ => {}
            }
        }

        Ok(Parameter {
            name,
            param_type,
            description,
        })
    }

    fn process_interaction_section(&self, node: &AstNode) -> Result<Vec<InteractionItem>> {
        let mut interactions = Vec::new();

        for child in &node.children {
            if child.node_type == AstNodeType::InteractionItem {
                let interaction_item = self.process_interaction_item(child)?;
                interactions.push(interaction_item);
            }
        }

        Ok(interactions)
    }

    fn process_interaction_item(&self, node: &AstNode) -> Result<InteractionItem> {
        for child in &node.children {
            match child.node_type {
                AstNodeType::StandardInteraction => {
                    let interaction = self.process_standard_interaction(child)?;
                    return Ok(InteractionItem::StandardInteraction(interaction));
                }
                AstNodeType::ProtocolComposition => {
                    let composition = self.process_protocol_composition(child)?;
                    return Ok(InteractionItem::ProtocolComposition(composition));
                }
                _ => {}
            }
        }

        Err(anyhow!(
            "No valid interaction type found in InteractionItem"
        ))
    }

    fn process_standard_interaction(&self, node: &AstNode) -> Result<StandardInteraction> {
        let mut from_role = "Unknown".to_string();
        let mut to_role = "Unknown".to_string();
        let mut action = "unknown_action".to_string();
        let mut description = "No description".to_string();
        let mut parameter_flows = Vec::new();

        for child in &node.children {
            match child.node_type {
                AstNodeType::RoleRef => {
                    if let Some(name) = child.get_string("name") {
                        if from_role == "Unknown" {
                            from_role = name.clone();
                        } else if to_role == "Unknown" {
                            to_role = name.clone();
                        }
                    }
                }
                AstNodeType::ActionName => {
                    if let Some(name) = child.get_string("name") {
                        action = name.clone();
                    }
                }
                AstNodeType::Annotation => {
                    if let Some(desc) = child.get_string("description") {
                        description = desc.clone();
                    }
                }
                AstNodeType::ParameterFlow => {
                    let param_flow = self.process_parameter_flow(child)?;
                    parameter_flows.push(param_flow);
                }
                _ => {}
            }
        }

        Ok(StandardInteraction {
            from_role,
            to_role,
            action,
            description,
            parameter_flows,
        })
    }

    fn process_protocol_composition(&self, node: &AstNode) -> Result<ProtocolComposition> {
        let mut protocol_name = "UnknownProtocol".to_string();
        let mut roles = Vec::new();
        let mut parameter_flows = Vec::new();

        for child in &node.children {
            match child.node_type {
                AstNodeType::ProtocolReference => {
                    // Extract protocol name from ProtocolReference
                    for ref_child in &child.children {
                        if ref_child.node_type == AstNodeType::Identifier {
                            if let Some(name) = ref_child.get_string("name") {
                                protocol_name = name.clone();
                            }
                        }
                    }
                }
                AstNodeType::Identifier => {
                    // This is a role identifier in the composition
                    if let Some(name) = child.get_string("name") {
                        roles.push(name.clone());
                    }
                }
                AstNodeType::ParameterFlow => {
                    let param_flow = self.process_parameter_flow(child)?;
                    parameter_flows.push(param_flow);
                }
                _ => {}
            }
        }

        Ok(ProtocolComposition {
            protocol_name,
            roles,
            parameter_flows,
        })
    }

    fn process_parameter_flow(&self, node: &AstNode) -> Result<ParameterFlow> {
        let mut direction = "unknown".to_string();
        let mut parameter = "unknown".to_string();

        if let Some(dir) = node.get_string("direction") {
            direction = dir.clone();
        }

        for child in &node.children {
            if child.node_type == AstNodeType::Identifier {
                if let Some(name) = child.get_string("name") {
                    parameter = name.clone();
                }
            }
        }

        Ok(ParameterFlow {
            direction,
            parameter,
        })
    }

    fn generate_rust_code(&self, protocols: &[Protocol]) -> Result<String> {
        let mut code = String::new();

        code.push_str("// Generated BMPP Protocol Implementation\n");
        code.push_str("use serde::{Serialize, Deserialize};\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use anyhow::Result;\n\n");

        // Generate Agent struct first
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        code.push_str("pub struct Agent {\n");
        code.push_str("    pub id: String,\n");
        code.push_str("    pub name: String,\n");
        code.push_str("}\n\n");

        // Generate code for each protocol
        for protocol in protocols {
            code.push_str(&self.generate_protocol_code(protocol)?);
        }

        Ok(code)
    }

    fn generate_protocol_code(&self, protocol: &Protocol) -> Result<String> {
        let mut code = String::new();

        // Generate protocol struct
        code.push_str(&format!(
            "/// {}\n#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct {}Protocol {{\n",
            protocol.description, protocol.name
        ));

        // Add roles as fields
        for role in &protocol.roles {
            code.push_str(&format!(
                "    /// {}\n    pub {}: Agent,\n",
                role.description,
                role.name.to_lowercase()
            ));
        }

        // Add parameters as fields
        for param in &protocol.parameters {
            let rust_type = self.map_bmpp_type_to_rust(&param.param_type);
            code.push_str(&format!(
                "    /// {}\n    pub {}: {},\n",
                param.description,
                param.name.to_lowercase(),
                rust_type
            ));
        }

        code.push_str("}\n\n");

        // Generate implementation
        code.push_str(&format!("impl {}Protocol {{\n", protocol.name));

        // Generate constructor
        code.push_str("    pub fn new() -> Self {\n");
        code.push_str("        Self {\n");
        for role in &protocol.roles {
            code.push_str(&format!(
                "            {}: Agent {{ id: String::new(), name: \"{}\".to_string() }},\n",
                role.name.to_lowercase(),
                role.name
            ));
        }
        for param in &protocol.parameters {
            let default_value = self.get_default_value(&param.param_type);
            code.push_str(&format!(
                "            {}: {},\n",
                param.name.to_lowercase(),
                default_value
            ));
        }
        code.push_str("        }\n");
        code.push_str("    }\n\n");

        // Generate methods for each interaction
        for interaction in &protocol.interactions {
            match interaction {
                InteractionItem::StandardInteraction(standard) => {
                    code.push_str(&self.generate_standard_interaction_method(standard, protocol)?);
                }
                InteractionItem::ProtocolComposition(composition) => {
                    code.push_str(&self.generate_composition_method(composition, protocol)?);
                }
            }
        }

        code.push_str("}\n\n");

        Ok(code)
    }

    fn generate_standard_interaction_method(
        &self,
        interaction: &StandardInteraction,
        protocol: &Protocol,
    ) -> Result<String> {
        let mut code = String::new();

        // Collect input and output parameters
        let mut input_params = Vec::new();
        let mut output_params = Vec::new();

        for param in &interaction.parameter_flows {
            if param.direction == "in" {
                input_params.push(&param.parameter);
            } else if param.direction == "out" {
                output_params.push(&param.parameter);
            }
        }

        // Generate method signature with proper types
        let method_name = interaction.action.to_lowercase();
        let mut signature = format!("    pub fn {}(&mut self", method_name);

        // Add input parameters
        for input_param in &input_params {
            let rust_type = self.get_parameter_type(input_param, protocol);
            signature.push_str(&format!(", {}: {}", input_param.to_lowercase(), rust_type));
        }

        // Determine return type
        let return_type = match output_params.len() {
            0 => "Result<()>".to_string(),
            1 => {
                let rust_type = self.get_parameter_type(&output_params[0], protocol);
                format!("Result<{}>", rust_type)
            }
            _ => {
                let types: Vec<String> = output_params
                    .iter()
                    .map(|param| self.get_parameter_type(param, protocol))
                    .collect();
                format!("Result<({})>", types.join(", "))
            }
        };

        signature.push_str(&format!(") -> {} {{", return_type));

        // Generate method
        code.push_str(&format!("    /// {}\n", interaction.description));
        code.push_str(&format!("{}\n", signature));
        code.push_str("        // Protocol interaction implementation\n");
        code.push_str(&format!(
            "        println!(\"Executing interaction: {} -> {} ({})\");\n",
            interaction.from_role, interaction.to_role, interaction.action
        ));

        // Log input parameters
        for input_param in &input_params {
            code.push_str(&format!(
                "        println!(\"Input parameter {}: {{:?}}\", {});\n",
                input_param,
                input_param.to_lowercase()
            ));
        }

        // Generate return value
        match output_params.len() {
            0 => code.push_str("        Ok(())\n"),
            1 => {
                let default = self.get_parameter_default(&output_params[0], protocol);
                code.push_str(&format!(
                    "        // Output parameter: {}\n",
                    output_params[0]
                ));
                code.push_str(&format!("        Ok({})\n", default));
            }
            _ => {
                let defaults: Vec<String> = output_params
                    .iter()
                    .map(|param| {
                        code.push_str(&format!("        // Output parameter: {}\n", param));
                        self.get_parameter_default(param, protocol)
                    })
                    .collect();
                code.push_str(&format!("        Ok(({}))\\n", defaults.join(", ")));
            }
        }

        code.push_str("    }\n\n");

        Ok(code)
    }

    fn generate_composition_method(
        &self,
        composition: &ProtocolComposition,
        protocol: &Protocol,
    ) -> Result<String> {
        let mut code = String::new();

        let method_name = format!("enact_{}", composition.protocol_name.to_lowercase());

        // Generate method signature
        code.push_str(&format!(
            "    /// Enacts the {} protocol with roles: {}\n",
            composition.protocol_name,
            composition.roles.join(", ")
        ));

        code.push_str(&format!(
            "    pub fn {}(&mut self) -> Result<()> {{\n",
            method_name
        ));
        code.push_str("        // Protocol composition enactment\n");
        code.push_str(&format!(
            "        println!(\"Enacting protocol: {} with roles: {}]\");\n",
            composition.protocol_name,
            composition.roles.join(", ")
        ));

        // Generate parameter flow logging
        for param_flow in &composition.parameter_flows {
            code.push_str(&format!(
                "        println!(\"Parameter flow: {} {}\");\n",
                param_flow.direction, param_flow.parameter
            ));
        }

        code.push_str("        Ok(())\n");
        code.push_str("    }\n\n");

        Ok(code)
    }

    fn get_parameter_type(&self, param_name: &str, protocol: &Protocol) -> String {
        protocol
            .parameters
            .iter()
            .find(|p| p.name == param_name)
            .map(|p| self.map_bmpp_type_to_rust(&p.param_type).to_string())
            .unwrap_or_else(|| "String".to_string())
    }

    fn get_parameter_default(&self, param_name: &str, protocol: &Protocol) -> String {
        protocol
            .parameters
            .iter()
            .find(|p| p.name == param_name)
            .map(|p| self.get_default_value(&p.param_type).to_string())
            .unwrap_or_else(|| "String::new()".to_string())
    }

    fn map_bmpp_type_to_rust(&self, bmpp_type: &str) -> &str {
        match bmpp_type {
            "String" => "String",
            "Int" => "i32",
            "Float" => "f64",
            "Bool" => "bool",
            _ => "String",
        }
    }

    fn get_default_value(&self, bmpp_type: &str) -> &str {
        match bmpp_type {
            "String" => "String::new()",
            "Int" => "0",
            "Float" => "0.0",
            "Bool" => "false",
            _ => "String::new()",
        }
    }
}
