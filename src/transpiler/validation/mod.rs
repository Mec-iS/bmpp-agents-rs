// src/validation/parameter_flow.rs
use crate::utils::ast::{AstNode, AstNodeType};
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};

pub mod errors;
mod tests;


/// Validates parameter flow consistency in BMPP protocols
pub fn validate_parameter_flow(ast: &AstNode) -> Result<()> {
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Expected Program node"));
    }

    for protocol_node in &ast.children {
        if protocol_node.node_type == AstNodeType::ProtocolDecl {
            validate_protocol_parameter_flow(protocol_node)?;
        }
    }

    Ok(())
}

fn validate_protocol_parameter_flow(protocol_node: &AstNode) -> Result<()> {
    let protocol_name = protocol_node.get_string("name").unwrap();
    
    // Collect all parameters and their information
    let mut parameters = HashMap::new();
    let mut interactions = Vec::new();
    
    // Extract parameters section
    for section in &protocol_node.children {
        match section.node_type {
            AstNodeType::ParametersSection => {
                for param_node in &section.children {
                    if param_node.node_type == AstNodeType::ParameterDecl {
                        let param_name = param_node.get_string("name").unwrap();
                        let param_type = param_node.get_string("type").unwrap();
                        parameters.insert(param_name.clone(), ParameterInfo {
                            name: param_name.clone(),
                            param_type: param_type.clone(),
                            producers: HashSet::new(),
                            consumers: HashSet::new(),
                        });
                    }
                }
            }
            AstNodeType::InteractionsSection => {
                for interaction_node in &section.children {
                    if interaction_node.node_type == AstNodeType::InteractionDecl {
                        interactions.push(extract_interaction_info(interaction_node)?);
                    }
                }
            }
            _ => {}
        }
    }
    
    // Build parameter flow graph
    for interaction in &interactions {
        // Process parameter flows
        for flow in &interaction.parameter_flows {
            if let Some(param_info) = parameters.get_mut(&flow.parameter) {
                match flow.direction.as_str() {
                    "out" => {
                        param_info.producers.insert(interaction.action.clone());
                    }
                    "in" => {
                        param_info.consumers.insert(interaction.action.clone());
                    }
                    _ => {
                        return Err(anyhow!(
                            "Invalid parameter direction '{}' for parameter '{}' in interaction '{}'", 
                            flow.direction, flow.parameter, interaction.action
                        ));
                    }
                }
            } else {
                return Err(anyhow!(
                    "Parameter '{}' used in interaction '{}' is not declared in protocol '{}'",
                    flow.parameter, interaction.action, protocol_name
                ));
            }
        }
    }
    
    // Validate flow consistency
    validate_flow_consistency(&parameters, protocol_name)?;
    validate_causality(&parameters, &interactions, protocol_name)?;
    
    Ok(())
}

#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    param_type: String,
    producers: HashSet<String>,
    consumers: HashSet<String>,
}

#[derive(Debug, Clone)]
struct InteractionInfo {
    action: String,
    from_role: String,
    to_role: String,
    parameter_flows: Vec<ParameterFlow>,
}

#[derive(Debug, Clone)]
struct ParameterFlow {
    direction: String,
    parameter: String,
}

fn extract_interaction_info(interaction_node: &AstNode) -> Result<InteractionInfo> {
    let action = interaction_node.get_string("action").unwrap().clone();
    let from_role = interaction_node.get_string("from_role").unwrap().clone();
    let to_role = interaction_node.get_string("to_role").unwrap().clone();
    
    let mut parameter_flows = Vec::new();
    
    for child in &interaction_node.children {
        if child.node_type == AstNodeType::ParameterFlow {
            for flow_child in &child.children {
                if flow_child.node_type == AstNodeType::ParameterRef {
                    let direction = flow_child.get_string("direction").unwrap().clone();
                    let parameter = flow_child.get_string("parameter").unwrap().clone();
                    
                    parameter_flows.push(ParameterFlow {
                        direction,
                        parameter,
                    });
                }
            }
        }
    }
    
    Ok(InteractionInfo {
        action,
        from_role,
        to_role,
        parameter_flows,
    })
}

fn validate_flow_consistency(parameters: &HashMap<String, ParameterInfo>, protocol_name: &str) -> Result<()> {
    for (param_name, param_info) in parameters {
        // Check that parameters with consumers have producers
        if !param_info.consumers.is_empty() && param_info.producers.is_empty() {
            // ID is a special parameter
            if param_name != "ID" {
                return Err(anyhow!(
                    "Parameter '{}' is consumed but never produced in protocol '{}'",
                    param_name, protocol_name
                ));
            }
        }
        
        // Check for multiple producers (safety violation)
        if param_info.producers.len() > 1 {
            return Err(anyhow!(
                "Parameter '{}' is produced by multiple interactions {:?} in protocol '{}' - this may cause safety violations",
                param_name, param_info.producers, protocol_name
            ));
        }
        
        // Warn about unused parameters
        if param_info.producers.is_empty() && param_info.consumers.is_empty() {
            println!("Warning: Parameter '{}' is declared but never used in protocol '{}'", 
                    param_name, protocol_name);
        }
    }
    
    Ok(())
}

fn validate_causality(
    parameters: &HashMap<String, ParameterInfo>,
    interactions: &[InteractionInfo],
    protocol_name: &str
) -> Result<()> {
    // Build dependency graph between interactions
    let mut interaction_deps: HashMap<String, HashSet<String>> = HashMap::new();
    
    for interaction in interactions {
        let mut deps = HashSet::new();
        
        // For each input parameter, find which interactions produce it
        for flow in &interaction.parameter_flows {
            if flow.direction == "in" {
                if let Some(param_info) = parameters.get(&flow.parameter) {
                    deps.extend(param_info.producers.iter().cloned());
                }
            }
        }
        
        interaction_deps.insert(interaction.action.clone(), deps);
    }
    
    // Check for cycles in the dependency graph
    for interaction in interactions {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        if has_cycle(&interaction.action, &interaction_deps, &mut visited, &mut path) {
            return Err(anyhow!(
                "Circular dependency detected in protocol '{}': {}",
                protocol_name,
                path.join(" -> ")
            ));
        }
    }
    
    Ok(())
}

fn has_cycle(
    current: &str,
    deps: &HashMap<String, HashSet<String>>,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>
) -> bool {
    if path.contains(&current.to_string()) {
        path.push(current.to_string());
        return true;
    }
    
    if visited.contains(current) {
        return false;
    }
    
    visited.insert(current.to_string());
    path.push(current.to_string());
    
    if let Some(dependencies) = deps.get(current) {
        for dep in dependencies {
            if has_cycle(dep, deps, visited, path) {
                return true;
            }
        }
    }
    
    path.pop();
    false
}
