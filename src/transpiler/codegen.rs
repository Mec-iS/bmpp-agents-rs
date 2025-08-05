use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::Result;
use serde::Serialize;

use crate::transpiler::validation::validate_parameter_flow;


#[derive(Serialize)]
struct Protocol {
    name: String,
    description: String,
    roles: Vec<Role>,
    parameters: Vec<Parameter>,
    interactions: Vec<Interaction>,
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
struct Interaction {
    from_role: String,
    to_role: String,
    action: String,
    description: String,
    parameter_flow: Vec<ParameterFlow>,
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
        validate_parameter_flow(ast)?;
        
        let mut protocols = Vec::new();
        
        for child in &ast.children {
            if child.node_type == AstNodeType::ProtocolDecl {
                let protocol = self.process_protocol(child)?;
                protocols.push(protocol);
            }
        }
        
        let generated_code = self.generate_rust_code(&protocols)?;
        Ok(generated_code)
    }
    
    fn process_protocol(&self, node: &AstNode) -> Result<Protocol> {
        let name = node.get_string("name").unwrap().clone();
        let description = node.get_string("description").unwrap().clone();
        
        let mut roles = Vec::new();
        let mut parameters = Vec::new();
        let mut interactions = Vec::new();
        
        for child in &node.children {
            match child.node_type {
                AstNodeType::RolesSection => {
                    roles = self.process_roles(child)?;
                },
                AstNodeType::ParametersSection => {
                    parameters = self.process_parameters(child)?;
                },
                AstNodeType::InteractionsSection => {
                    interactions = self.process_interactions(child)?;
                },
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
                let name = child.get_string("name").unwrap().clone();
                let description = child.get_string("description").unwrap().clone();
                roles.push(Role { name, description });
            }
        }
        
        Ok(roles)
    }
    
    fn process_parameters(&self, node: &AstNode) -> Result<Vec<Parameter>> {
        let mut parameters = Vec::new();
        
        for child in &node.children {
            if child.node_type == AstNodeType::ParameterDecl {
                let name = child.get_string("name").unwrap().clone();
                let param_type = child.get_string("type").unwrap().clone();
                let description = child.get_string("description").unwrap().clone();
                
                parameters.push(Parameter {
                    name,
                    param_type,
                    description,
                });
            }
        }
        
        Ok(parameters)
    }
    
    fn process_interactions(&self, node: &AstNode) -> Result<Vec<Interaction>> {
        let mut interactions = Vec::new();
        
        for child in &node.children {
            if child.node_type == AstNodeType::InteractionDecl {
                let from_role = child.get_string("from_role").unwrap().clone();
                let to_role = child.get_string("to_role").unwrap().clone();
                let action = child.get_string("action").unwrap().clone();
                let description = child.get_string("description").unwrap().clone();
                
                let mut parameter_flow = Vec::new();
                
                for param_child in &child.children {
                    if param_child.node_type == AstNodeType::ParameterFlow {
                        for flow_child in &param_child.children {
                            if flow_child.node_type == AstNodeType::ParameterRef {
                                let direction = flow_child.get_string("direction").unwrap().clone();
                                let parameter = flow_child.get_string("parameter").unwrap().clone();
                                
                                parameter_flow.push(ParameterFlow {
                                    direction,
                                    parameter,
                                });
                            }
                        }
                    }
                }
                
                interactions.push(Interaction {
                    from_role,
                    to_role,
                    action,
                    description,
                    parameter_flow,
                });
            }
        }
        
        Ok(interactions)
    }
    
    fn generate_rust_code(&self, protocols: &[Protocol]) -> Result<String> {
        let mut code = String::new();
        
        code.push_str("// Generated BMPP Protocol Implementation\n");
        code.push_str("use serde::{Serialize, Deserialize};\n");
        code.push_str("use std::collections::HashMap;\n");
        code.push_str("use anyhow::Result;\n\n");
        
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
        
        // Generate Agent struct
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        code.push_str("pub struct Agent {\n    pub id: String,\n    pub name: String,\n}\n\n");
        
        // Generate interaction methods
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
        
        for interaction in &protocol.interactions {
            code.push_str(&self.generate_interaction_method(interaction, protocol)?);
        }
        
        code.push_str("}\n\n");
        
        Ok(code)
    }
    
    fn generate_interaction_method(&self, interaction: &Interaction, protocol: &Protocol) -> Result<String> {
        let mut code = String::new();
        
        // Collect input and output parameters
        let mut input_params = Vec::new();
        let mut output_params = Vec::new();
        
        for param in &interaction.parameter_flow {
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
                .unwrap_or("String") // fallback
                .to_string()
        };
        
        // Helper closure to get default value for a parameter
        let get_param_default = |param_name: &str| -> String {
            protocol.parameters.iter()
                .find(|p| p.name == param_name)
                .map(|p| self.get_default_value(&p.param_type))
                .unwrap_or("String::new()") // fallback
                .to_string()
        };
        
        // Generate function signature with input parameters
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
        code.push_str("        // Protocol interaction implementation\n");
        code.push_str(&format!(
            "        println!(\"Executing interaction: {} -> {} ({})\");\n",
            interaction.from_role,
            interaction.to_role,
            interaction.action
        ));
        
        // Add comments for input parameters with their values
        for input_param in &input_params {
            code.push_str(&format!(
                "        println!(\"Input parameter {}: {{:?}}\", {});\n", 
                input_param,
                input_param.to_lowercase()
            ));
        }
        
        // Add comments for output parameters
        for output_param in &output_params {
            code.push_str(&format!(
                "        // Output parameter: {}\n",
                output_param
            ));
        }
        
        // Generate return statement based on output parameters
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
    
    fn map_bmpp_type_to_rust(&self, bmpp_type: &str) -> &str {
        match bmpp_type {
            "String" => "String",
            "Int" => "i32",
            "Float" => "f64",
            "Bool" => "bool",
            _ => "String", // Default fallback
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
