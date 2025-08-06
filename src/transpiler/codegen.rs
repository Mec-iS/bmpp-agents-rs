use crate::protocol::ast::{AstNode, AstNodeType};
use anyhow::{anyhow, Result};
use serde::Serialize;
use crate::transpiler::composition::ProtocolRegistry;

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

pub struct BmppCodeGenerator {
    registry: ProtocolRegistry,
}

impl BmppCodeGenerator {
    pub fn new() -> Self {
        Self {
            registry: ProtocolRegistry::new(),
        }
    }
    
    pub fn generate(&self, ast: &AstNode) -> Result<String> {
        match ast.node_type {
            AstNodeType::Program => self.generate_program(ast),
            _ => Err(anyhow!("Expected Program node, got {:?}", ast.node_type)),
        }
    }

    fn generate_program(&self, ast: &AstNode) -> Result<String> {
        let mut protocols = Vec::new();
        
        // First pass: collect all protocols
        let mut registry = ProtocolRegistry::new();
        self.collect_protocols(ast, &mut registry)?;
        
        // Second pass: process each protocol
        for protocol_node in &ast.children {
            if protocol_node.node_type == AstNodeType::Protocol {
                let mut protocol_copy = (**protocol_node).clone();
                registry.resolve_protocol_references(&mut protocol_copy)?;
                let protocol = self.process_protocol(&protocol_copy)?;
                protocols.push(protocol);
            }
        }
        
        if protocols.is_empty() {
            return Err(anyhow!("No protocols found in AST"));
        }
        
        self.generate_rust_code(&protocols)
    }

    fn collect_protocols(&self, node: &AstNode, registry: &mut ProtocolRegistry) -> Result<()> {
        if node.node_type == AstNodeType::Protocol {
            // Extract protocol name from ProtocolName child node
            if let Some(protocol_name_node) = node.children.iter()
                .find(|child| child.node_type == AstNodeType::ProtocolName) {
                if let Some(name) = protocol_name_node.get_string("name") {
                    registry.register_protocol(name.clone(), node.clone());
                }
            }
        }
        
        // Recursively collect from children
        for child in &node.children {
            self.collect_protocols(child, registry)?;
        }
        
        Ok(())
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
        
        Err(anyhow!("No valid interaction type found in InteractionItem"))
    }
    
    fn process_standard_interaction(&self, node: &AstNode) -> Result<StandardInteraction> {
        let mut from_role = "Unknown".to_string();
        let mut to_role = "Unknown".to_string();
        let mut action = "unknown_action".to_string();
        let mut description = "No description".to_string();
        let mut parameter_flows = Vec::new();
        
        for child in &node.children {
            match child.node_type {
                AstNodeType::Identifier => {
                    if let Some(role_type) = child.get_string("role") {
                        if let Some(name) = child.get_string("name") {
                            match role_type.as_str() {
                                "from" => from_role = name.clone(),
                                "to" => to_role = name.clone(),
                                "action" => action = name.clone(),
                                _ => {}
                            }
                        }
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
        code.push_str("use anyhow::Result;\n");
        code.push_str("use std::sync::Arc;\n");
        code.push_str("use std::marker::PhantomData;\n\n");
        
        // Generate Agent struct first
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        code.push_str("pub struct Agent {\n    pub id: String,\n    pub name: String,\n}\n\n");
        
        // Generate protocol enactment trait
        code.push_str("pub trait ProtocolEnactment<P, C> {\n");
        code.push_str("    fn enact(&self, parent: &P, roles: &[String], params: &HashMap<String, String>) -> Result<C>;\n");
        code.push_str("}\n\n");
        
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
        code.push_str("        }\n    }\n\n");
        
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
    
    fn generate_standard_interaction_method(&self, interaction: &StandardInteraction, protocol: &Protocol) -> Result<String> {
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
        
        // Helper closure to get parameter type from protocol definition
        let get_param_type = |param_name: &str| -> String {
            protocol.parameters.iter()
                .find(|p| p.name == param_name)
                .map(|p| self.map_bmpp_type_to_rust(&p.param_type))
                .unwrap_or("String")
                .to_string()
        };
        
        // Helper closure to get default value for a parameter
        let get_param_default = |param_name: &str| -> String {
            protocol.parameters.iter()
                .find(|p| p.name == param_name)
                .map(|p| self.get_default_value(&p.param_type))
                .unwrap_or("String::new()")
                .to_string()
        };
        
        // Generate function signature
        let mut signature = format!("    pub fn {}(&mut self", interaction.action.to_lowercase());
        
        // Add input parameters to signature
        for input_param in &input_params {
            let rust_type = get_param_type(input_param);
            signature.push_str(&format!(", {}: {}", input_param.to_lowercase(), rust_type));
        }
        
        // Determine return type based on output parameters
        let return_type = match output_params.len() {
            0 => "Result<()>".to_string(),
            1 => {
                let rust_type = get_param_type(&output_params[0]);
                format!("Result<{}>", rust_type)
            }
            _ => {
                let output_types: Vec<String> = output_params.iter()
                    .map(|param| get_param_type(param))
                    .collect();
                format!("Result<({})>", output_types.join(", "))
            }
        };
        
        signature.push_str(&format!(") -> {} {{", return_type));
        
        // Generate method documentation and signature
        code.push_str(&format!(
            "    /// {}\n{}\n",
            interaction.description,
            signature
        ));
        
        // Generate method body
        code.push_str("        // Standard interaction implementation\n");
        code.push_str(&format!(
            "        println!(\"Executing interaction: {} -> {} ({})\");\n",
            interaction.from_role,
            interaction.to_role,
            interaction.action
        ));
        
        // Add comments for input parameters
        for input_param in &input_params {
            code.push_str(&format!(
                "        println!(\"Input parameter {}: {{:?}}\", {});\n", 
                input_param,
                input_param.to_lowercase()
            ));
        }
        
        // Generate return statement
        match output_params.len() {
            0 => code.push_str("        Ok(())\n"),
            1 => {
                let default_value = get_param_default(&output_params[0]);
                code.push_str(&format!("        Ok({})\n", default_value));
            }
            _ => {
                let default_values: Vec<String> = output_params.iter()
                    .map(|param| get_param_default(param))
                    .collect();
                code.push_str(&format!("        Ok(({}))\n", default_values.join(", ")));
            }
        }
        
        code.push_str("    }\n\n");
        
        Ok(code)
    }
    
    fn generate_composition_method(&self, composition: &ProtocolComposition, protocol: &Protocol) -> Result<String> {
        let mut code = String::new();
        
        // Collect input and output parameters
        let mut input_params = Vec::new();
        let mut output_params = Vec::new();
        
        for param in &composition.parameter_flows {
            if param.direction == "in" {
                input_params.push(&param.parameter);
            } else if param.direction == "out" {
                output_params.push(&param.parameter);
            }
        }
        
        // Helper closure to get parameter type from protocol definition
        let get_param_type = |param_name: &str| -> String {
            protocol.parameters.iter()
                .find(|p| p.name == param_name)
                .map(|p| self.map_bmpp_type_to_rust(&p.param_type))
                .unwrap_or("String")
                .to_string()
        };
        
        // Generate function signature
        let method_name = format!("enact_{}", composition.protocol_name.to_lowercase());
        let mut signature = format!("    pub fn {}(&mut self", method_name);
        
        // Add input parameters to signature
        for input_param in &input_params {
            let rust_type = get_param_type(input_param);
            signature.push_str(&format!(", {}: {}", input_param.to_lowercase(), rust_type));
        }
        
        // Determine return type based on output parameters
        let return_type = match output_params.len() {
            0 => format!("Result<{}Protocol>", composition.protocol_name),
            1 => {
                let rust_type = get_param_type(&output_params[0]);
                format!("Result<({}, {})>", rust_type, format!("{}Protocol", composition.protocol_name))
            }
            _ => {
                let output_types: Vec<String> = output_params.iter()
                    .map(|param| get_param_type(param))
                    .collect();
                format!("Result<({}, {})>", 
                    output_types.join(", "), 
                    format!("{}Protocol", composition.protocol_name))
            }
        };
        
        signature.push_str(&format!(") -> {} {{", return_type));
        
        // Generate method documentation and signature
        code.push_str(&format!(
            "    /// Enacts the {} protocol with roles: {}\n{}\n",
            composition.protocol_name,
            composition.roles.join(", "),
            signature
        ));
        
        // Generate method body
        code.push_str("        // Protocol composition enactment\n");
        code.push_str(&format!(
            "        println!(\"Enacting protocol: {}\");\n",
            composition.protocol_name
        ));
        
        // Create role mapping
        code.push_str("        let mut role_mapping = HashMap::new();\n");
        for (index, role) in composition.roles.iter().enumerate() {
            code.push_str(&format!(
                "        role_mapping.insert(\"{}\".to_string(), \"{}\".to_string());\n",
                index, role
            ));
        }
        
        // Create parameter mapping
        code.push_str("        let mut param_mapping = HashMap::new();\n");
        for input_param in &input_params {
            code.push_str(&format!(
                "        param_mapping.insert(\"{}\".to_string(), format!(\"{{:?}}\", {}));\n",
                input_param, input_param.to_lowercase()
            ));
        }
        
        // Create and configure child protocol
        code.push_str(&format!(
            "        let mut child_protocol = {}Protocol::new();\n",
            composition.protocol_name
        ));
        
        // Configure child protocol roles based on parent roles
        for (index, role) in composition.roles.iter().enumerate() {
            if index < composition.roles.len() {
                code.push_str(&format!(
                    "        child_protocol.{} = self.{}.clone();\n",
                    role.to_lowercase(),
                    role.to_lowercase()
                ));
            }
        }
        
        // Set input parameters on child protocol
        for input_param in &input_params {
            code.push_str(&format!(
                "        child_protocol.{} = {};\n",
                input_param.to_lowercase(),
                input_param.to_lowercase()
            ));
        }
        
        // Generate return statement
        match output_params.len() {
            0 => code.push_str("        Ok(child_protocol)\n"),
            _ => {
                // For now, return default values for output parameters along with child protocol
                let default_values: Vec<String> = output_params.iter()
                    .map(|param| {
                        let default_string = "String".to_string();
                        let param_type = protocol.parameters.iter()
                            .find(|p| p.name == **param)
                            .map(|p| &p.param_type)
                            .unwrap_or(&default_string);
                        self.get_default_value(param_type).to_string()
                    })
                    .collect();
                
                if output_params.len() == 1 {
                    code.push_str(&format!("        Ok(({}, child_protocol))\n", default_values[0]));
                } else {
                    code.push_str(&format!("        Ok((({}, ), child_protocol))\n", default_values.join(", ")));
                }
            }
        }
        
        code.push_str("    }\n\n");
        
        Ok(code)
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
