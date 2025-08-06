use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq)]
pub enum AstNodeType {
    Program,
    Protocol,
    ProtocolName,
    Annotation,
    RolesSection,
    RoleDecl,
    ParametersSection,
    ParameterDecl,
    InteractionSection,
    InteractionItem,
    StandardInteraction,
    ProtocolComposition,
    ProtocolReference,
    Identifier,
    BasicType,
    ParameterFlow,
    RoleRef,
    ActionName,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_type: AstNodeType,
    pub children: Vec<Box<AstNode>>,
    pub properties: HashMap<String, String>,
    pub line: usize,
    pub column: usize,
    pub parent: Option<*const AstNode>,
}

impl AstNode {
    pub fn new(node_type: AstNodeType) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            properties: HashMap::new(),
            line: 0,
            column: 0,
            parent: None,
        }
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    pub fn get_string(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

impl AstNode {
    fn format_node_name(&self) -> String {
        let node_type_str = format!("{:?}", self.node_type);
        
        // Get the primary property value for display
        let property_value = self.get_primary_property_value();
        
        if let Some(value) = property_value {
            format!("{} (\"{}\")", node_type_str, value)
        } else {
            node_type_str
        }
    }

    fn get_primary_property_value(&self) -> Option<&String> {
        match self.node_type {
            AstNodeType::ProtocolName => self.get_string("name"),
            AstNodeType::Annotation => self.get_string("description"),
            AstNodeType::Identifier => self.get_string("name"),
            AstNodeType::RoleRef => self.get_string("name"),
            AstNodeType::ActionName => self.get_string("name"),
            AstNodeType::BasicType => self.get_string("type"),
            AstNodeType::ParameterFlow => {
                // For parameter flows, show direction and parameter name
                if let (Some(direction), Some(param_name)) = 
                    (self.get_string("direction"), self.children.first()?.get_string("name")) {
                    // Create a temporary string - we'll need to handle this differently
                    // For now, just show the direction
                    Some(direction)
                } else {
                    self.get_string("direction")
                }
            }
            AstNodeType::ProtocolReference => {
                // For protocol references, get the name from the first Identifier child
                self.children.first()?.get_string("name")
            }
            _ => None,
        }
    }
}

// Enhanced display implementation for parameter flows
impl AstNode {
    pub fn format_parameter_flow(&self) -> Option<String> {
        if self.node_type == AstNodeType::ParameterFlow {
            if let Some(direction) = self.get_string("direction") {
                if let Some(child) = self.children.first() {
                    if let Some(param_name) = child.get_string("name") {
                        return Some(format!("{}, {}", direction, param_name));
                    }
                }
                return Some(direction.clone());
            }
        }
        None
    }
}

// Override the display for specific node types that need special formatting
impl AstNode {
    fn format_node_name_enhanced(&self) -> String {
        let node_type_str = format!("{:?}", self.node_type);
        
        match self.node_type {
            AstNodeType::ParameterFlow => {
                if let Some(formatted) = self.format_parameter_flow() {
                    format!("{} (\"{}\")", node_type_str, formatted)
                } else {
                    node_type_str
                }
            }
            AstNodeType::ProtocolReference => {
                if let Some(child) = self.children.first() {
                    if let Some(name) = child.get_string("name") {
                        format!("{} (\"{}\")", node_type_str, name)
                    } else {
                        node_type_str
                    }
                } else {
                    node_type_str
                }
            }
            _ => {
                let property_value = self.get_primary_property_value();
                if let Some(value) = property_value {
                    format!("{} (\"{}\")", node_type_str, value)
                } else {
                    node_type_str
                }
            }
        }
    }
}

// Update the display_tree method to use the enhanced formatting
impl AstNode {
    fn display_tree(
        &self,
        f: &mut Formatter<'_>,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) -> FmtResult {
        // Print the current node
        if is_root {
            write!(f, "{}", self.format_node_name_enhanced())?;
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            writeln!(f, "{}{}{}", prefix, connector, self.format_node_name_enhanced())?;
        }

        // Calculate prefix for children
        let child_prefix = if is_root {
            String::new()
        } else {
            let extension = if is_last { "    " } else { "│   " };
            format!("{}{}", prefix, extension)
        };

        // Print children
        let child_count = self.children.len();
        for (index, child) in self.children.iter().enumerate() {
            let is_last_child = index == child_count - 1;
            
            if is_root && index == 0 {
                writeln!(f)?; // Add newline after root node
            }
            
            child.display_tree(f, &child_prefix, is_last_child, false)?;
        }

        Ok(())
    }
}

// Update the Display implementation to use enhanced formatting
impl Display for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.display_tree(f, "", true, true)
    }
}

// Debug print function (keeping the existing functionality)
pub fn _print_ast_debug(node: &AstNode, max_depth: usize) {
    fn print_recursive(node: &AstNode, depth: usize, max_depth: usize) {
        if depth > max_depth {
            return;
        }
        
        let indent = "  ".repeat(depth);
        println!("{}AstNode {{ node_type: {:?}, children: [", indent, node.node_type);
        
        if !node.properties.is_empty() {
            println!("{}  properties: {{", indent);
            for (key, value) in &node.properties {
                println!("{}    \"{}\": {:?}", indent, key, value);
            }
            println!("{}  }}", indent);
        }
        
        for child in &node.children {
            print_recursive(child, depth + 1, max_depth);
        }
        
        println!("{}]", indent);
        if depth == 0 {
            println!("}}");
        }
    }
    
    print_recursive(node, 0, max_depth);
}

// Convenience methods for testing and debugging
impl AstNode {
    /// Pretty print the AST tree to stdout
    pub fn print_tree(&self) {
        println!("{}", self);
    }
    
    /// Get a formatted string representation of the tree
    pub fn to_tree_string(&self) -> String {
        format!("{}", self)
    }
    
    /// Find all nodes of a specific type
    pub fn find_nodes(&self, node_type: AstNodeType) -> Vec<AstNode> {
        let mut result = Vec::new();
        self.collect_nodes_of_type(node_type, &mut result);
        result
    }
    
    fn collect_nodes_of_type(&self, target_type: AstNodeType, result: &mut Vec<AstNode>) {
        if self.node_type == target_type {
            result.push(self.clone());
        }
        
        for child in &self.children {
            child.collect_nodes_of_type(target_type.clone(), result);
        }
    }
    
    /// Count nodes of a specific type
    pub fn count_nodes(&self, node_type: AstNodeType) -> usize {
        self.find_nodes(node_type).len()
    }
}

impl AstNode {
    // ... existing methods ...

    /// Find the first child node of a specific type
    pub fn find_child(&self, node_type: AstNodeType) -> Option<&AstNode> {
        self.children.iter()
            .find(|child| child.node_type == node_type)
            .map(|boxed_child| boxed_child.as_ref())
    }

    /// Find the first mutable child node of a specific type
    pub fn find_child_mut(&mut self, node_type: AstNodeType) -> Option<&mut AstNode> {
        self.children.iter_mut()
            .find(|child| child.node_type == node_type)
            .map(|boxed_child| boxed_child.as_mut())
    }

    /// Find all child nodes of a specific type
    pub fn find_children(&self, node_type: AstNodeType) -> Vec<&AstNode> {
        self.children.iter()
            .filter(|child| child.node_type == node_type)
            .map(|boxed_child| boxed_child.as_ref())
            .collect()
    }

    /// Get the protocol name from a Protocol node
    pub fn get_protocol_name(&self) -> Option<String> {
        if self.node_type != AstNodeType::Protocol {
            return None;
        }
        self.find_child(AstNodeType::ProtocolName)?
            .get_string("name")
            .cloned()
    }

    /// Get the protocol annotation from a Protocol node
    pub fn get_protocol_annotation(&self) -> Option<String> {
        if self.node_type != AstNodeType::Protocol {
            return None;
        }
        self.find_child(AstNodeType::Annotation)?
            .get_string("description")
            .cloned()
    }

    /// Get the roles section from a Protocol node
    pub fn get_roles_section(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::Protocol {
            return None;
        }
        self.find_child(AstNodeType::RolesSection)
    }

    /// Get the parameters section from a Protocol node
    pub fn get_parameters_section(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::Protocol {
            return None;
        }
        self.find_child(AstNodeType::ParametersSection)
    }

    /// Get the interactions section from a Protocol node
    pub fn get_interactions_section(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::Protocol {
            return None;
        }
        self.find_child(AstNodeType::InteractionSection)
    }

    /// Get all role declarations from a RolesSection
    pub fn get_role_declarations(&self) -> Vec<&AstNode> {
        if self.node_type != AstNodeType::RolesSection {
            return Vec::new();
        }
        self.find_children(AstNodeType::RoleDecl)
    }

    /// Get all parameter declarations from a ParametersSection
    pub fn get_parameter_declarations(&self) -> Vec<&AstNode> {
        if self.node_type != AstNodeType::ParametersSection {
            return Vec::new();
        }
        self.find_children(AstNodeType::ParameterDecl)
    }

    /// Get all interaction items from an InteractionSection
    pub fn get_interaction_items(&self) -> Vec<&AstNode> {
        if self.node_type != AstNodeType::InteractionSection {
            return Vec::new();
        }
        self.find_children(AstNodeType::InteractionItem)
    }

    /// Get the standard interaction from an InteractionItem (if it contains one)
    pub fn get_standard_interaction(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::InteractionItem {
            return None;
        }
        self.find_child(AstNodeType::StandardInteraction)
    }

    /// Get the protocol composition from an InteractionItem (if it contains one)
    pub fn get_protocol_composition(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::InteractionItem {
            return None;
        }
        self.find_child(AstNodeType::ProtocolComposition)
    }

    /// Get all parameter flows from a StandardInteraction or ProtocolComposition
    pub fn get_parameter_flows(&self) -> Vec<&AstNode> {
        match self.node_type {
            AstNodeType::StandardInteraction | AstNodeType::ProtocolComposition => {
                self.find_children(AstNodeType::ParameterFlow)
            }
            _ => Vec::new()
        }
    }

    /// Get the role references from a StandardInteraction
    pub fn get_role_refs(&self) -> Vec<&AstNode> {
        if self.node_type != AstNodeType::StandardInteraction {
            return Vec::new();
        }
        self.find_children(AstNodeType::RoleRef)
    }

    /// Get the action name from a StandardInteraction
    pub fn get_action_name(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::StandardInteraction {
            return None;
        }
        self.find_child(AstNodeType::ActionName)
    }

    /// Get the protocol reference from a ProtocolComposition
    pub fn get_protocol_reference(&self) -> Option<&AstNode> {
        if self.node_type != AstNodeType::ProtocolComposition {
            return None;
        }
        self.find_child(AstNodeType::ProtocolReference)
    }

    /// Get the identifier name from various node types
    pub fn get_identifier_name(&self) -> Option<String> {
        match self.node_type {
            AstNodeType::Identifier | AstNodeType::RoleRef | AstNodeType::ActionName => {
                self.get_string("name").cloned()
            }
            AstNodeType::ProtocolName => {
                self.get_string("name").cloned()
            }
            AstNodeType::ProtocolReference => {
                // For ProtocolReference, get name from child Identifier
                self.find_child(AstNodeType::Identifier)?
                    .get_string("name")
                    .cloned()
            }
            _ => None
        }
    }

    /// Get parameter flow information (direction and parameter name)
    pub fn get_parameter_flow_info(&self) -> Option<(String, String)> {
        if self.node_type != AstNodeType::ParameterFlow {
            return None;
        }
        
        let direction = self.get_string("direction")?.clone();
        let param_name = self.find_child(AstNodeType::Identifier)?
            .get_string("name")?
            .clone();
        
        Some((direction, param_name))
    }

    /// Get role declaration information (name and description)
    pub fn get_role_decl_info(&self) -> Option<(String, String)> {
        if self.node_type != AstNodeType::RoleDecl {
            return None;
        }
        
        let name = self.find_child(AstNodeType::Identifier)?
            .get_string("name")?
            .clone();
        let description = self.find_child(AstNodeType::Annotation)?
            .get_string("description")?
            .clone();
        
        Some((name, description))
    }

    /// Get parameter declaration information (name, type, and description)
    pub fn get_parameter_decl_info(&self) -> Option<(String, String, String)> {
        if self.node_type != AstNodeType::ParameterDecl {
            return None;
        }
        
        let name = self.find_child(AstNodeType::Identifier)?
            .get_string("name")?
            .clone();
        let param_type = self.find_child(AstNodeType::BasicType)?
            .get_string("type")?
            .clone();
        let description = self.find_child(AstNodeType::Annotation)?
            .get_string("description")?
            .clone();
        
        Some((name, param_type, description))
    }

    /// Get standard interaction information
    pub fn get_standard_interaction_info(&self) -> Option<StandardInteractionInfo> {
        if self.node_type != AstNodeType::StandardInteraction {
            return None;
        }
        
        let role_refs = self.get_role_refs();
        let from_role = role_refs.get(0)?.get_identifier_name()?;
        let to_role = role_refs.get(1)?.get_identifier_name()?;
        
        let action_name = self.get_action_name()?.get_identifier_name()?;
        let description = self.find_child(AstNodeType::Annotation)?
            .get_string("description")?
            .clone();
        
        let parameter_flows = self.get_parameter_flows()
            .iter()
            .filter_map(|pf| pf.get_parameter_flow_info())
            .collect();
        
        Some(StandardInteractionInfo {
            from_role,
            to_role,
            action_name,
            description,
            parameter_flows,
        })
    }
}

/// Helper struct for standard interaction information
#[derive(Debug, Clone)]
pub struct StandardInteractionInfo {
    pub from_role: String,
    pub to_role: String,
    pub action_name: String,
    pub description: String,
    pub parameter_flows: Vec<(String, String)>, // (direction, parameter_name)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let mut node = AstNode::new(AstNodeType::Protocol);
        node.set_string("name", "TestProtocol");
        
        assert_eq!(node.node_type, AstNodeType::Protocol);
        assert_eq!(node.get_string("name"), Some(&"TestProtocol".to_string()));
    }

    #[test]
    fn test_tree_display() {
        // Create a simple AST for testing
        let mut program = AstNode::new(AstNodeType::Program);
        
        let mut protocol = AstNode::new(AstNodeType::Protocol);
        
        let mut protocol_name = AstNode::new(AstNodeType::ProtocolName);
        protocol_name.set_string("name", "TestProtocol");
        
        let mut annotation = AstNode::new(AstNodeType::Annotation);
        annotation.set_string("description", "A test protocol");
        
        protocol.children.push(Box::new(protocol_name));
        protocol.children.push(Box::new(annotation));
        
        program.children.push(Box::new(protocol));
        
        let tree_string = program.to_tree_string();
        
        // Test that the output contains expected elements
        assert!(tree_string.contains("Program"));
        assert!(tree_string.contains("Protocol"));
        assert!(tree_string.contains("ProtocolName (\"TestProtocol\")"));
        assert!(tree_string.contains("Annotation (\"A test protocol\")"));
        assert!(tree_string.contains("└──"));
        assert!(tree_string.contains("├──"));
    }

    #[test]
    fn test_node_finding() {
        let mut program = AstNode::new(AstNodeType::Program);
        
        let mut protocol = AstNode::new(AstNodeType::Protocol);
        let annotation1 = AstNode::new(AstNodeType::Annotation);
        let annotation2 = AstNode::new(AstNodeType::Annotation);
        
        protocol.children.push(Box::new(annotation1));
        protocol.children.push(Box::new(annotation2));
        program.children.push(Box::new(protocol));
        
        let annotations = program.find_nodes(AstNodeType::Annotation);
        assert_eq!(annotations.len(), 2);
        
        let protocols = program.find_nodes(AstNodeType::Protocol);
        assert_eq!(protocols.len(), 1);
        
        assert_eq!(program.count_nodes(AstNodeType::Annotation), 2);
    }
}
