use crate::protocol::ast::{AstNode, AstNodeType};
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ProtocolRegistry {
    protocols: HashMap<String, AstNode>,
}

#[derive(Debug, Clone)]
pub struct CompositionParameter {
    pub parameter_type: CompositionParameterType,
    pub name: String,
    pub direction: Option<String>, // Some("in"/"out") for parameter flows, None for role identifiers
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositionParameterType {
    RoleIdentifier,
    ParameterFlow,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        Self {
            protocols: HashMap::new(),
        }
    }
    
    /// Register a protocol in the registry for later reference
    pub fn register_protocol(&mut self, name: String, protocol: AstNode) {
        self.protocols.insert(name, protocol);
    }
    
    /// Build protocol registry from a Program AST node
    pub fn from_program(program: &AstNode) -> Result<Self> {
        let mut registry = Self::new();
        
        if program.node_type != AstNodeType::Program {
            return Err(anyhow!("Expected Program node, got {:?}", program.node_type));
        }
        
        for protocol_node in &program.children {
            if protocol_node.node_type == AstNodeType::Protocol {
                let protocol_name = Self::extract_protocol_name(protocol_node)?;
                registry.register_protocol(protocol_name, (**protocol_node).clone());
            }
        }
        
        Ok(registry)
    }
    
    /// Extract protocol name from Protocol AST node
    fn extract_protocol_name(protocol_node: &AstNode) -> Result<String> {
        for child in &protocol_node.children {
            if child.node_type == AstNodeType::ProtocolName {
                if let Some(name) = child.get_string("name") {
                    return Ok(name.clone());
                }
            }
        }
        Err(anyhow!("Protocol name not found"))
    }
    
    /// Resolve all protocol references in a protocol AST
    pub fn resolve_protocol_references(&self, protocol: &mut AstNode) -> Result<()> {
        self.resolve_references_recursive(protocol)?;
        Ok(())
    }
    
    fn resolve_references_recursive(&self, node: &mut AstNode) -> Result<()> {
        match node.node_type {
            AstNodeType::ProtocolComposition => {
                self.resolve_composition(node)?;
            }
            _ => {
                // Recursively process child nodes
                for child in &mut node.children {
                    self.resolve_references_recursive(child)?;
                }
            }
        }
        Ok(())
    }
    
    /// Resolve a single protocol composition
    fn resolve_composition(&self, composition_node: &mut AstNode) -> Result<()> {
        // Extract the protocol reference name
        let protocol_name = self.extract_composition_protocol_name(composition_node)?;
        
        // Find the referenced protocol
        let referenced_protocol = self.protocols.get(&protocol_name)
            .ok_or_else(|| anyhow!("Undefined protocol: {}", protocol_name))?;
        
        // Extract composition parameters
        let composition_params = self.extract_composition_parameters(composition_node)?;
        
        // Validate composition parameters against the referenced protocol
        self.validate_composition_parameters(referenced_protocol, &composition_params, &protocol_name)?;
        
        // Create an instance of the referenced protocol with parameter bindings
        let instance = self.create_protocol_instance(
            referenced_protocol, 
            &composition_params,
            &protocol_name
        )?;
        
        // Replace the composition node with the expanded instance
        *composition_node = instance;
        
        Ok(())
    }
    
    /// Extract protocol name from ProtocolComposition node
    fn extract_composition_protocol_name(&self, composition_node: &AstNode) -> Result<String> {
        for child in &composition_node.children {
            if child.node_type == AstNodeType::ProtocolReference {
                // ProtocolReference contains an Identifier child with the protocol name
                for ref_child in &child.children {
                    if ref_child.node_type == AstNodeType::Identifier {
                        if let Some(name) = ref_child.get_string("name") {
                            return Ok(name.clone());
                        }
                    }
                }
            }
        }
        Err(anyhow!("Protocol reference name not found in composition"))
    }
    
    /// Extract composition parameters from ProtocolComposition node
    fn extract_composition_parameters(&self, composition_node: &AstNode) -> Result<Vec<CompositionParameter>> {
        let mut parameters = Vec::new();
        
        for child in &composition_node.children {
            match child.node_type {
                AstNodeType::Identifier => {
                    // Role identifier
                    if let Some(name) = child.get_string("name") {
                        parameters.push(CompositionParameter {
                            parameter_type: CompositionParameterType::RoleIdentifier,
                            name: name.clone(),
                            direction: None,
                        });
                    }
                }
                AstNodeType::ParameterFlow => {
                    // Parameter flow (in/out parameter)
                    if let Some(direction) = child.get_string("direction") {
                        // Extract parameter name from the Identifier child
                        for flow_child in &child.children {
                            if flow_child.node_type == AstNodeType::Identifier {
                                if let Some(param_name) = flow_child.get_string("name") {
                                    parameters.push(CompositionParameter {
                                        parameter_type: CompositionParameterType::ParameterFlow,
                                        name: param_name.clone(),
                                        direction: Some(direction.clone()),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
                AstNodeType::ProtocolReference => {
                    // Skip - handled separately
                }
                _ => {
                    return Err(anyhow!("Unexpected child node type in composition: {:?}", child.node_type));
                }
            }
        }
        
        Ok(parameters)
    }
    
    /// Validate composition parameters against the referenced protocol
    fn validate_composition_parameters(
        &self,
        referenced_protocol: &AstNode,
        composition_params: &[CompositionParameter],
        protocol_name: &str
    ) -> Result<()> {
        // Extract protocol's declared roles and parameters
        let protocol_roles = self.extract_protocol_roles(referenced_protocol);
        let protocol_parameters = self.extract_protocol_parameters(referenced_protocol);
        
        // Count role identifiers and parameter flows in composition
        let role_count = composition_params.iter()
            .filter(|p| p.parameter_type == CompositionParameterType::RoleIdentifier)
            .count();
            
        let param_flow_count = composition_params.iter()
            .filter(|p| p.parameter_type == CompositionParameterType::ParameterFlow)
            .count();
        
        // Validate role count
        if role_count != protocol_roles.len() {
            return Err(anyhow!(
                "Role count mismatch in composition of '{}': expected {}, got {}",
                protocol_name, protocol_roles.len(), role_count
            ));
        }
        
        // Validate that all parameter flows reference declared parameters
        for param in composition_params {
            if param.parameter_type == CompositionParameterType::ParameterFlow {
                if !protocol_parameters.contains(&param.name) {
                    return Err(anyhow!(
                        "Parameter '{}' in composition of '{}' is not declared in the referenced protocol",
                        param.name, protocol_name
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract role names from a protocol
    fn extract_protocol_roles(&self, protocol: &AstNode) -> Vec<String> {
        let mut roles = Vec::new();
        
        for child in &protocol.children {
            if child.node_type == AstNodeType::RolesSection {
                for role_child in &child.children {
                    if role_child.node_type == AstNodeType::RoleDecl {
                        // RoleDecl contains an Identifier child with the role name
                        for role_grandchild in &role_child.children {
                            if role_grandchild.node_type == AstNodeType::Identifier {
                                if let Some(role_name) = role_grandchild.get_string("name") {
                                    roles.push(role_name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        roles
    }
    
    /// Extract parameter names from a protocol
    fn extract_protocol_parameters(&self, protocol: &AstNode) -> Vec<String> {
        let mut parameters = Vec::new();
        
        for child in &protocol.children {
            if child.node_type == AstNodeType::ParametersSection {
                for param_child in &child.children {
                    if param_child.node_type == AstNodeType::ParameterDecl {
                        // ParameterDecl contains an Identifier child with the parameter name
                        for param_grandchild in &param_child.children {
                            if param_grandchild.node_type == AstNodeType::Identifier {
                                if let Some(param_name) = param_grandchild.get_string("name") {
                                    parameters.push(param_name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        parameters
    }
    
    /// Create an instance of a referenced protocol with parameter bindings
    fn create_protocol_instance(
        &self,
        referenced_protocol: &AstNode,
        composition_params: &[CompositionParameter],
        protocol_name: &str
    ) -> Result<AstNode> {
        let mut instance = referenced_protocol.clone();
        
        // Create role mappings
        let role_mappings = self.create_role_mappings(&instance, composition_params)?;
        
        // Create parameter mappings
        let parameter_mappings = self.create_parameter_mappings(composition_params);
        
        // Apply role and parameter mappings to the instance
        self.apply_mappings(&mut instance, &role_mappings, &parameter_mappings)?;
        
        // Recursively resolve any nested references in the instance
        self.resolve_references_recursive(&mut instance)?;
        
        Ok(instance)
    }
    
    /// Create role name mappings for protocol instantiation
    fn create_role_mappings(
        &self,
        protocol: &AstNode,
        composition_params: &[CompositionParameter]
    ) -> Result<HashMap<String, String>> {
        let protocol_roles = self.extract_protocol_roles(protocol);
        let composition_roles: Vec<&str> = composition_params.iter()
            .filter(|p| p.parameter_type == CompositionParameterType::RoleIdentifier)
            .map(|p| p.name.as_str())
            .collect();
        
        if protocol_roles.len() != composition_roles.len() {
            return Err(anyhow!(
                "Role count mismatch: protocol has {}, composition has {}",
                protocol_roles.len(), composition_roles.len()
            ));
        }
        
        let mappings: HashMap<String, String> = protocol_roles.iter()
            .zip(composition_roles.iter())
            .map(|(protocol_role, composition_role)| {
                (protocol_role.clone(), composition_role.to_string())
            })
            .collect();
        
        Ok(mappings)
    }
    
    /// Create parameter name mappings for protocol instantiation
    fn create_parameter_mappings(&self, composition_params: &[CompositionParameter]) -> HashMap<String, String> {
        // For now, parameters keep their original names
        // In a more sophisticated implementation, this could handle parameter renaming
        composition_params.iter()
            .filter(|p| p.parameter_type == CompositionParameterType::ParameterFlow)
            .map(|p| (p.name.clone(), p.name.clone()))
            .collect()
    }
    
    /// Apply role and parameter mappings to a protocol instance
    fn apply_mappings(
        &self,
        node: &mut AstNode,
        role_mappings: &HashMap<String, String>,
        parameter_mappings: &HashMap<String, String>
    ) -> Result<()> {
        match node.node_type {
            AstNodeType::RoleRef => {
                // Update role references in interactions
                if let Some(role_name) = node.get_string("name") {
                    if let Some(new_role) = role_mappings.get(role_name) {
                        node.set_string("name", new_role);
                    }
                }
            }
            AstNodeType::Identifier => {
                // Update identifier nodes that reference roles or parameters
                if let Some(name) = node.get_string("name") {
                    // Check if it's a role reference
                    if let Some(new_role) = role_mappings.get(name) {
                        node.set_string("name", new_role);
                    }
                    // Check if it's a parameter reference
                    else if let Some(new_param) = parameter_mappings.get(name) {
                        node.set_string("name", new_param);
                    }
                }
            }
            AstNodeType::ParameterFlow => {
                // Update parameter flows
                for child in &mut node.children {
                    if child.node_type == AstNodeType::Identifier {
                        if let Some(param_name) = child.get_string("name") {
                            if let Some(new_param) = parameter_mappings.get(param_name) {
                                child.set_string("name", new_param);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        // Recursively apply mappings to children
        for child in &mut node.children {
            self.apply_mappings(child, role_mappings, parameter_mappings)?;
        }
        
        Ok(())
    }
    
    /// Get all registered protocol names
    pub fn get_protocol_names(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }
    
    /// Check if a protocol is registered
    pub fn has_protocol(&self, name: &str) -> bool {
        self.protocols.contains_key(name)
    }
    
    /// Get a protocol by name
    pub fn get_protocol(&self, name: &str) -> Option<&AstNode> {
        self.protocols.get(name)
    }
}

/// Validate all protocol compositions in a program
pub fn validate_protocol_compositions(program: &AstNode) -> Result<()> {
    let registry = ProtocolRegistry::from_program(program)?;
    
    // Check all protocols for composition references
    for protocol_node in &program.children {
        if protocol_node.node_type == AstNodeType::Protocol {
            validate_protocol_compositions_recursive(protocol_node, &registry)?;
        }
    }
    
    Ok(())
}

fn validate_protocol_compositions_recursive(node: &AstNode, registry: &ProtocolRegistry) -> Result<()> {
    match node.node_type {
        AstNodeType::ProtocolComposition => {
            // Extract and validate protocol reference
            let protocol_name = extract_composition_protocol_name_for_validation(node)?;
            
            if !registry.has_protocol(&protocol_name) {
                return Err(anyhow!("Unknown protocol '{}' referenced in composition", protocol_name));
            }
            
            // Validate composition parameters
            let composition_params = extract_composition_parameters_for_validation(node)?;
            let referenced_protocol = registry.get_protocol(&protocol_name).unwrap();
            
            registry.validate_composition_parameters(referenced_protocol, &composition_params, &protocol_name)?;
        }
        _ => {
            // Recursively validate children
            for child in &node.children {
                validate_protocol_compositions_recursive(child, registry)?;
            }
        }
    }
    
    Ok(())
}

fn extract_composition_protocol_name_for_validation(composition_node: &AstNode) -> Result<String> {
    for child in &composition_node.children {
        if child.node_type == AstNodeType::ProtocolReference {
            for ref_child in &child.children {
                if ref_child.node_type == AstNodeType::Identifier {
                    if let Some(name) = ref_child.get_string("name") {
                        return Ok(name.clone());
                    }
                }
            }
        }
    }
    Err(anyhow!("Protocol reference name not found in composition"))
}

fn extract_composition_parameters_for_validation(composition_node: &AstNode) -> Result<Vec<CompositionParameter>> {
    let mut parameters = Vec::new();
    
    for child in &composition_node.children {
        match child.node_type {
            AstNodeType::Identifier => {
                if let Some(name) = child.get_string("name") {
                    parameters.push(CompositionParameter {
                        parameter_type: CompositionParameterType::RoleIdentifier,
                        name: name.clone(),
                        direction: None,
                    });
                }
            }
            AstNodeType::ParameterFlow => {
                if let Some(direction) = child.get_string("direction") {
                    for flow_child in &child.children {
                        if flow_child.node_type == AstNodeType::Identifier {
                            if let Some(param_name) = flow_child.get_string("name") {
                                parameters.push(CompositionParameter {
                                    parameter_type: CompositionParameterType::ParameterFlow,
                                    name: param_name.clone(),
                                    direction: Some(direction.clone()),
                                });
                                break;
                            }
                        }
                    }
                }
            }
            AstNodeType::ProtocolReference => {
                // Skip - handled separately
            }
            _ => {}
        }
    }
    
    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_registry_creation() {
        let registry = ProtocolRegistry::new();
        assert_eq!(registry.get_protocol_names().len(), 0);
    }

    #[test]
    fn test_composition_parameter_types() {
        let role_param = CompositionParameter {
            parameter_type: CompositionParameterType::RoleIdentifier,
            name: "TestRole".to_string(),
            direction: None,
        };
        
        let flow_param = CompositionParameter {
            parameter_type: CompositionParameterType::ParameterFlow,
            name: "TestParam".to_string(),
            direction: Some("in".to_string()),
        };
        
        assert_eq!(role_param.parameter_type, CompositionParameterType::RoleIdentifier);
        assert_eq!(flow_param.parameter_type, CompositionParameterType::ParameterFlow);
        assert_eq!(flow_param.direction, Some("in".to_string()));
    }
}
