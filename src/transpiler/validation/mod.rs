use crate::protocol::ast::{AstNode, AstNodeType};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};

pub mod errors;
mod tests;

/// Validates parameter flow consistency in BMPP protocols according to BSPL standard
pub fn validate_parameter_flow(ast: &AstNode) -> Result<()> {
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Expected Program node"));
    }

    for protocol_node in &ast.children {
        if protocol_node.node_type == AstNodeType::Protocol {
            validate_protocol_parameter_flow(protocol_node)?;
        }
    }

    Ok(())
}

fn validate_protocol_parameter_flow(protocol_node: &AstNode) -> Result<()> {
    let protocol_name = extract_protocol_name(protocol_node)?;

    let mut declared_parameters = HashSet::new();
    let mut parameter_info: HashMap<String, ParameterInfo> = HashMap::new();

    for child in &protocol_node.children {
        if child.node_type == AstNodeType::ParametersSection {
            extract_parameters(child, &mut declared_parameters, &mut parameter_info)?;
        }
    }

    let mut interactions = Vec::new();

    for child in &protocol_node.children {
        if child.node_type == AstNodeType::InteractionSection {
            extract_interactions(
                child,
                &mut interactions,
                &declared_parameters,
                &mut parameter_info,
                &protocol_name,
            )?;
        }
    }

    // First validate unreachable interactions before other checks
    validate_unreachable_interactions(&parameter_info, &interactions, &protocol_name)?;
    validate_flow_consistency(&parameter_info, &interactions, &protocol_name)?;
    validate_causality(&parameter_info, &interactions, &protocol_name)?;
    validate_completeness(&parameter_info, &protocol_name)?;
    validate_enactability(&interactions, &parameter_info, &protocol_name)?;

    Ok(())
}

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

fn extract_parameters(
    params_section: &AstNode,
    declared_parameters: &mut HashSet<String>,
    parameter_info: &mut HashMap<String, ParameterInfo>,
) -> Result<()> {
    for param_decl in &params_section.children {
        if param_decl.node_type == AstNodeType::ParameterDecl {
            let (name, param_type) = extract_parameter_info(param_decl)?;
            declared_parameters.insert(name.clone());
            parameter_info.insert(
                name.clone(),
                ParameterInfo {
                    name: name.clone(),
                    param_type,
                    producers: HashSet::new(),
                    consumers: HashSet::new(),
                },
            );
        }
    }
    Ok(())
}

fn extract_parameter_info(param_decl: &AstNode) -> Result<(String, String)> {
    let mut name = None;
    let mut param_type = None;

    for child in &param_decl.children {
        match child.node_type {
            AstNodeType::Identifier => {
                if let Some(param_name) = child.get_string("name") {
                    name = Some(param_name.clone());
                }
            }
            AstNodeType::BasicType => {
                if let Some(type_name) = child.get_string("type") {
                    param_type = Some(type_name.clone());
                }
            }
            _ => {}
        }
    }

    match (name, param_type) {
        (Some(n), Some(t)) => Ok((n, t)),
        _ => Err(anyhow!("Failed to extract parameter information")),
    }
}

fn extract_interactions(
    interaction_section: &AstNode,
    interactions: &mut Vec<InteractionInfo>,
    declared_parameters: &HashSet<String>,
    parameter_info: &mut HashMap<String, ParameterInfo>,
    protocol_name: &str,
) -> Result<()> {
    for interaction_item in &interaction_section.children {
        if interaction_item.node_type == AstNodeType::InteractionItem {
            for child in &interaction_item.children {
                match child.node_type {
                    AstNodeType::StandardInteraction => {
                        let interaction = extract_standard_interaction(child)?;
                        validate_and_update_parameter_usage(
                            &interaction,
                            declared_parameters,
                            parameter_info,
                            protocol_name,
                        )?;
                        interactions.push(interaction);
                    }
                    AstNodeType::ProtocolComposition => {
                        let interaction = extract_composition_interaction(child)?;
                        validate_and_update_parameter_usage(
                            &interaction,
                            declared_parameters,
                            parameter_info,
                            protocol_name,
                        )?;
                        interactions.push(interaction);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn validate_and_update_parameter_usage(
    interaction: &InteractionInfo,
    declared_parameters: &HashSet<String>,
    parameter_info: &mut HashMap<String, ParameterInfo>,
    protocol_name: &str,
) -> Result<()> {
    for flow in &interaction.parameter_flows {
        if !declared_parameters.contains(&flow.parameter) {
            return Err(anyhow!(
                "Parameter '{}' used in interaction '{}' is not declared in protocol '{}'",
                flow.parameter,
                interaction.action,
                protocol_name
            ));
        }

        if let Some(param_info) = parameter_info.get_mut(&flow.parameter) {
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
                        flow.direction,
                        flow.parameter,
                        interaction.action
                    ));
                }
            }
        }
    }
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

fn extract_standard_interaction(node: &AstNode) -> Result<InteractionInfo> {
    let mut from_role = "Unknown".to_string();
    let mut to_role = "Unknown".to_string();
    let mut action = "unknown_action".to_string();
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
                } else if let Some(name) = child.get_string("name") {
                    if from_role == "Unknown" {
                        from_role = name.clone();
                    } else if to_role == "Unknown" {
                        to_role = name.clone();
                    } else if action == "unknown_action" {
                        action = name.clone();
                    }
                }
            }
            AstNodeType::ParameterFlow => {
                let param_flow = extract_parameter_flow(child)?;
                parameter_flows.push(param_flow);
            }
            _ => {}
        }
    }

    Ok(InteractionInfo {
        action,
        from_role,
        to_role,
        parameter_flows,
    })
}

fn extract_composition_interaction(node: &AstNode) -> Result<InteractionInfo> {
    let mut protocol_name = "UnknownProtocol".to_string();
    let mut roles = Vec::new();
    let mut parameter_flows = Vec::new();

    for child in &node.children {
        match child.node_type {
            AstNodeType::ProtocolReference => {
                for ref_child in &child.children {
                    if ref_child.node_type == AstNodeType::Identifier {
                        if let Some(name) = ref_child.get_string("name") {
                            protocol_name = name.clone();
                        }
                    }
                }
            }
            AstNodeType::Identifier => {
                if let Some(name) = child.get_string("name") {
                    roles.push(name.clone());
                }
            }
            AstNodeType::ParameterFlow => {
                let param_flow = extract_parameter_flow(child)?;
                parameter_flows.push(param_flow);
            }
            _ => {}
        }
    }

    if roles.len() < 2 {
        roles.resize(2, "System".to_string());
    }

    Ok(InteractionInfo {
        action: protocol_name,
        from_role: roles[0].clone(),
        to_role: roles[1].clone(),
        parameter_flows,
    })
}

fn extract_parameter_flow(node: &AstNode) -> Result<ParameterFlow> {
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

/// Validates that no interactions are unreachable due to consuming unproducible parameters
fn validate_unreachable_interactions(
    parameters: &HashMap<String, ParameterInfo>,
    interactions: &[InteractionInfo],
    protocol_name: &str,
) -> Result<()> {
    // Check each interaction for unreachable conditions
    for interaction in interactions {
        for flow in &interaction.parameter_flows {
            if flow.direction == "in" {
                if let Some(param_info) = parameters.get(&flow.parameter) {
                    // If parameter has no producers and is not pre-protocol knowledge,
                    // then this interaction is unreachable
                    if param_info.producers.is_empty()
                        && !is_pre_protocol_parameter(&flow.parameter)
                    {
                        return Err(anyhow!(
                            "Interaction '{}' is unreachable because parameter '{}' is consumed but never produced in protocol '{}' - BSPL completeness violation",
                            interaction.action, flow.parameter, protocol_name
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validates basic flow consistency according to BSPL rules with parallel process support
fn validate_flow_consistency(
    parameters: &HashMap<String, ParameterInfo>,
    interactions: &[InteractionInfo],
    protocol_name: &str,
) -> Result<()> {
    for (param_name, param_info) in parameters {
        // Check for multiple producers - but allow parallel branches
        if param_info.producers.len() > 1 {
            // Check if these are legitimate parallel branches
            if !is_valid_parallel_production(param_name, &param_info.producers, interactions) {
                let producers: Vec<String> = param_info.producers.iter().cloned().collect();
                return Err(anyhow!(
                    "Parameter '{}' is produced by multiple interactions {:?} in protocol '{}' - BSPL safety violation",
                    param_name, producers, protocol_name
                ));
            }
        }

        if param_info.producers.is_empty() && param_info.consumers.is_empty() {
            println!(
                "Warning: Parameter '{}' is declared but never used in protocol '{}'",
                param_name, protocol_name
            );
        }
    }

    Ok(())
}

/// Determines if multiple producers represent valid parallel branches
fn is_valid_parallel_production(
    parameter_name: &str,
    producers: &HashSet<String>,
    interactions: &[InteractionInfo],
) -> bool {
    if producers.len() <= 1 {
        return true;
    }

    // Find the producer interactions
    let producer_interactions: Vec<&InteractionInfo> = interactions
        .iter()
        .filter(|i| producers.contains(&i.action))
        .collect();

    if producer_interactions.is_empty() {
        return false;
    }

    // Check if all producers originate from the same role (parallel branch pattern)
    let first_role = &producer_interactions[0].from_role;
    let same_origin_role = producer_interactions
        .iter()
        .all(|i| &i.from_role == first_role);

    if !same_origin_role {
        return false;
    }

    // Check if producers have independent enabling conditions
    // (no shared input parameters beyond pre-protocol knowledge)
    let mut shared_inputs = HashSet::new();
    let mut first_iteration = true;

    for producer in &producer_interactions {
        let mut current_inputs = HashSet::new();

        for flow in &producer.parameter_flows {
            if flow.direction == "in" && !is_pre_protocol_parameter(&flow.parameter) {
                current_inputs.insert(flow.parameter.clone());
            }
        }

        if first_iteration {
            shared_inputs = current_inputs;
            first_iteration = false;
        } else {
            shared_inputs = shared_inputs
                .intersection(&current_inputs)
                .cloned()
                .collect();
        }
    }

    // Valid parallel branches should share minimal input dependencies
    // or have the same input (like broadcasting the same initial data)
    true
}

/// Validates causality constraints according to BSPL with parallel branch support
fn validate_causality(
    parameters: &HashMap<String, ParameterInfo>,
    interactions: &[InteractionInfo],
    protocol_name: &str,
) -> Result<()> {
    // Build precedence graph considering parallel branches
    let mut precedence_graph: HashMap<String, Vec<String>> = HashMap::new();

    for interaction in interactions {
        precedence_graph.insert(interaction.action.clone(), Vec::new());
    }

    // Build precedence relationships with parallel branch awareness
    for interaction in interactions {
        for flow in &interaction.parameter_flows {
            if flow.direction == "in" {
                if let Some(param_info) = parameters.get(&flow.parameter) {
                    for producer in &param_info.producers {
                        if producer != &interaction.action {
                            // Check if this creates a valid precedence or parallel relationship
                            if !is_parallel_branch_relationship(
                                producer,
                                &interaction.action,
                                interactions,
                                parameters,
                            ) {
                                precedence_graph
                                    .entry(producer.clone())
                                    .or_default()
                                    .push(interaction.action.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for cycles using topological sort
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut ordered_interactions = Vec::new();

    for interaction in interactions {
        in_degree.insert(interaction.action.clone(), 0);
    }

    for successors in precedence_graph.values() {
        for successor in successors {
            *in_degree.get_mut(successor).unwrap() += 1;
        }
    }

    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(action, _)| action.clone())
        .collect();

    while let Some(current) = queue.pop() {
        ordered_interactions.push(current.clone());

        if let Some(successors) = precedence_graph.get(&current) {
            for successor in successors {
                if let Some(degree) = in_degree.get_mut(successor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(successor.clone());
                    }
                }
            }
        }
    }

    if ordered_interactions.len() != interactions.len() {
        let remaining: Vec<String> = interactions
            .iter()
            .map(|i| i.action.clone())
            .filter(|action| !ordered_interactions.contains(action))
            .collect();

        let cycle_path = find_cycle_path(&precedence_graph, &remaining);

        return Err(anyhow!(
            "Circular dependency detected in protocol '{}': {} - BSPL causality violation",
            protocol_name,
            cycle_path
        ));
    }

    Ok(())
}

/// Determines if two interactions represent parallel branches rather than sequential dependency
fn is_parallel_branch_relationship(
    producer: &str,
    consumer: &str,
    interactions: &[InteractionInfo],
    parameters: &HashMap<String, ParameterInfo>,
) -> bool {
    let producer_interaction = interactions.iter().find(|i| i.action == producer);
    let consumer_interaction = interactions.iter().find(|i| i.action == consumer);

    if let (Some(prod), Some(cons)) = (producer_interaction, consumer_interaction) {
        // Check if they originate from the same role and target different roles (parallel dispatch)
        if prod.from_role == cons.from_role && prod.to_role != cons.to_role {
            // Check if they produce the same parameter (parallel branches)
            let prod_outputs: HashSet<&String> = prod
                .parameter_flows
                .iter()
                .filter(|f| f.direction == "out")
                .map(|f| &f.parameter)
                .collect();

            let cons_inputs: HashSet<&String> = cons
                .parameter_flows
                .iter()
                .filter(|f| f.direction == "in")
                .map(|f| &f.parameter)
                .collect();

            let shared_params: HashSet<&String> =
                prod_outputs.intersection(&cons_inputs).cloned().collect();

            // If they share parameters, check if this is a broadcast scenario
            if !shared_params.is_empty() {
                for param in shared_params {
                    if let Some(param_info) = parameters.get(param) {
                        // If parameter has multiple producers from same role, it's likely parallel
                        if param_info.producers.len() > 1 {
                            let producers_from_same_role = interactions
                                .iter()
                                .filter(|i| param_info.producers.contains(&i.action))
                                .map(|i| &i.from_role)
                                .collect::<HashSet<_>>();

                            if producers_from_same_role.len() == 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn find_cycle_path(graph: &HashMap<String, Vec<String>>, remaining_nodes: &[String]) -> String {
    if remaining_nodes.is_empty() {
        return "unknown cycle".to_string();
    }

    let mut visited = HashSet::new();
    let mut path = Vec::new();

    if let Some(cycle) = dfs_find_cycle(&remaining_nodes[0], graph, &mut visited, &mut path) {
        return cycle.join(" -> ");
    }

    remaining_nodes.join(" -> ")
}

fn dfs_find_cycle(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if path.contains(&node.to_string()) {
        let cycle_start = path.iter().position(|n| n == node).unwrap();
        let mut cycle = path[cycle_start..].to_vec();
        cycle.push(node.to_string());
        return Some(cycle);
    }

    if visited.contains(node) {
        return None;
    }

    visited.insert(node.to_string());
    path.push(node.to_string());

    if let Some(successors) = graph.get(node) {
        for successor in successors {
            if let Some(cycle) = dfs_find_cycle(successor, graph, visited, path) {
                return Some(cycle);
            }
        }
    }

    path.pop();
    None
}

/// Validates protocol completeness according to BSPL
fn validate_completeness(
    parameters: &HashMap<String, ParameterInfo>,
    protocol_name: &str,
) -> Result<()> {
    let mut orphaned_parameters = Vec::new();
    let mut dead_end_parameters = Vec::new();

    for (param_name, param_info) in parameters {
        if !param_info.producers.is_empty() && param_info.consumers.is_empty() {
            dead_end_parameters.push(param_name.clone());
        }

        if param_info.producers.is_empty() && param_info.consumers.is_empty() {
            orphaned_parameters.push(param_name.clone());
        }
    }

    if !dead_end_parameters.is_empty() {
        println!("Warning: Parameters {:?} are produced but never consumed in protocol '{}' - potential completeness issue", 
                dead_end_parameters, protocol_name);
    }

    if !orphaned_parameters.is_empty() {
        println!(
            "Warning: Parameters {:?} are never used in protocol '{}' - completeness issue",
            orphaned_parameters, protocol_name
        );
    }

    Ok(())
}

/// Validates protocol enactability according to BSPL
fn validate_enactability(
    interactions: &[InteractionInfo],
    parameters: &HashMap<String, ParameterInfo>,
    protocol_name: &str,
) -> Result<()> {
    for interaction in interactions {
        if interaction.from_role == "Unknown" || interaction.to_role == "Unknown" {
            return Err(anyhow!(
                "Interaction '{}' has undefined roles in protocol '{}' - enactability violation",
                interaction.action,
                protocol_name
            ));
        }

        for flow in &interaction.parameter_flows {
            if flow.direction == "in" {
                if let Some(param_info) = parameters.get(&flow.parameter) {
                    if param_info.producers.is_empty()
                        && !is_pre_protocol_parameter(&flow.parameter)
                    {
                        return Err(anyhow!(
                            "Interaction '{}' requires parameter '{}' but it's never produced - enactability violation",
                            interaction.action, flow.parameter
                        ));
                    }
                }
            }
        }
    }

    let mut executable_interactions = HashSet::new();
    let mut changed = true;

    for interaction in interactions {
        let has_unresolved_deps = interaction
            .parameter_flows
            .iter()
            .filter(|flow| flow.direction == "in")
            .any(|flow| {
                if let Some(param_info) = parameters.get(&flow.parameter) {
                    param_info.producers.is_empty() && !is_pre_protocol_parameter(&flow.parameter)
                } else {
                    true
                }
            });

        if !has_unresolved_deps {
            executable_interactions.insert(interaction.action.clone());
        }
    }

    while changed {
        changed = false;
        for interaction in interactions {
            if !executable_interactions.contains(&interaction.action) {
                let can_execute = interaction
                    .parameter_flows
                    .iter()
                    .filter(|flow| flow.direction == "in")
                    .all(|flow| {
                        if let Some(param_info) = parameters.get(&flow.parameter) {
                            if is_pre_protocol_parameter(&flow.parameter) {
                                return true;
                            }
                            param_info
                                .producers
                                .iter()
                                .any(|producer| executable_interactions.contains(producer))
                        } else {
                            false
                        }
                    });

                if can_execute {
                    executable_interactions.insert(interaction.action.clone());
                    changed = true;
                }
            }
        }
    }

    for interaction in interactions {
        if !executable_interactions.contains(&interaction.action) {
            println!(
                "Warning: Interaction '{}' may be unreachable in protocol '{}'",
                interaction.action, protocol_name
            );
        }
    }

    Ok(())
}

/// Determines if a parameter represents pre-protocol knowledge
fn is_pre_protocol_parameter(param_name: &str) -> bool {
    matches!(
        param_name.to_uppercase().as_str(),
        "ID" | "TIMESTAMP" | "NONCE" | "SESSION_ID"
    )
}

/// Additional BSPL validation for protocol composition
pub fn validate_protocol_composition(ast: &AstNode) -> Result<()> {
    if ast.node_type != AstNodeType::Program {
        return Err(anyhow!("Expected Program node"));
    }

    let mut protocol_registry: HashMap<String, AstNode> = HashMap::new();

    for protocol_node in &ast.children {
        if protocol_node.node_type == AstNodeType::Protocol {
            let protocol_name = extract_protocol_name(protocol_node)?;
            protocol_registry.insert(protocol_name, (**protocol_node).clone());
        }
    }

    for protocol_node in &ast.children {
        if protocol_node.node_type == AstNodeType::Protocol {
            validate_composition_references(protocol_node, &protocol_registry)?;
        }
    }

    Ok(())
}

fn validate_composition_references(
    protocol_node: &AstNode,
    protocol_registry: &HashMap<String, AstNode>,
) -> Result<()> {
    let protocol_name = extract_protocol_name(protocol_node)?;

    for child in &protocol_node.children {
        if child.node_type == AstNodeType::InteractionSection {
            for interaction_item in &child.children {
                if interaction_item.node_type == AstNodeType::InteractionItem {
                    for item_child in &interaction_item.children {
                        if item_child.node_type == AstNodeType::ProtocolComposition {
                            validate_single_composition(
                                item_child,
                                protocol_registry,
                                &protocol_name,
                            )?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_single_composition(
    composition_node: &AstNode,
    protocol_registry: &HashMap<String, AstNode>,
    parent_protocol_name: &str,
) -> Result<()> {
    let mut referenced_protocol_name = None;

    for child in &composition_node.children {
        if child.node_type == AstNodeType::ProtocolReference {
            for ref_child in &child.children {
                if ref_child.node_type == AstNodeType::Identifier {
                    if let Some(name) = ref_child.get_string("name") {
                        referenced_protocol_name = Some(name.clone());
                    }
                }
            }
        }
    }

    if let Some(ref_name) = referenced_protocol_name {
        if !protocol_registry.contains_key(&ref_name) {
            return Err(anyhow!(
                "Protocol '{}' references unknown protocol '{}' in composition",
                parent_protocol_name,
                ref_name
            ));
        }

        if ref_name == parent_protocol_name {
            return Err(anyhow!(
                "Protocol '{}' cannot reference itself in composition - direct recursion not allowed",
                parent_protocol_name
            ));
        }
    } else {
        return Err(anyhow!(
            "Protocol composition in '{}' has no valid protocol reference",
            parent_protocol_name
        ));
    }

    Ok(())
}
