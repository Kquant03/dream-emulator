// src-tauri/engine/src/compiler/mod.rs
use crate::{VisualScript, VisualScriptNode, Project};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CompiledSystem {
    pub name: String,
    pub code: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("Unknown node type: {0}")]
    UnknownNode(String),
    
    #[error("Invalid connection: {0}")]
    InvalidConnection(String),
    
    #[error("Code generation failed: {0}")]
    CodeGeneration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn compile_visual_script(script: &VisualScript) -> Result<CompiledSystem, CompilerError> {
    let mut compiler = ScriptCompiler::new();
    compiler.compile(script)
}

struct ScriptCompiler {
    code: Vec<String>,
    indent_level: usize,
    temp_vars: HashMap<String, String>,
    var_counter: usize,
}

impl ScriptCompiler {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            indent_level: 0,
            temp_vars: HashMap::new(),
            var_counter: 0,
        }
    }
    
    fn compile(&mut self, script: &VisualScript) -> Result<CompiledSystem, CompilerError> {
        // Generate imports
        self.write_line("use dream_engine::{World, PhysicsWorld, System, EntityId};");
        self.write_line("use dream_engine::{Transform, Sprite, RigidBody, Vec2, Vec3};");
        self.write_line("");
        
        // Generate system struct
        let system_name = self.to_rust_name(&script.name);
        self.write_line(&format!("pub struct {}System {{", system_name));
        self.indent();
        self.write_line("// System state");
        self.dedent();
        self.write_line("}");
        self.write_line("");
        
        // Generate system implementation
        self.write_line(&format!("impl System for {}System {{", system_name));
        self.indent();
        
        self.write_line("fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32) {");
        self.indent();
        
        // Sort nodes topologically
        let sorted_nodes = self.topological_sort(&script.nodes, &script.connections)?;
        
        // Compile each node
        for node in sorted_nodes {
            self.compile_node(node)?;
        }
        
        self.dedent();
        self.write_line("}");
        
        self.dedent();
        self.write_line("}");
        
        Ok(CompiledSystem {
            name: script.name.clone(),
            code: self.code.join("\n"),
        })
    }
    
    fn compile_node(&mut self, node: &VisualScriptNode) -> Result<(), CompilerError> {
        self.write_line(&format!("// {}", node.data.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(node.get_type())));
        
        match node.get_type() {
            "event/update" => {
                // Update event is implicit in the execute method
                self.temp_vars.insert(format!("{}_dt", node.id), "dt".to_string());
            }
            
            "event/collision" => {
                self.write_line("for event in physics.get_collision_events() {");
                self.indent();
                self.temp_vars.insert(format!("{}_entity_a", node.id), "event.entity_a".to_string());
                self.temp_vars.insert(format!("{}_entity_b", node.id), "event.entity_b".to_string());
                self.temp_vars.insert(format!("{}_contact", node.id), "event.contact".to_string());
            }
            
            "query/get_entities" => {
                let components = node.data.get("components")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>())
                    .unwrap_or_else(|| vec!["Transform"]);
                
                let query_var = self.gen_var("query");
                let component_refs = components.iter()
                    .map(|c| format!("&{}", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                self.write_line(&format!(
                    "for (entity, ({})) in world.query::<({})>().iter() {{",
                    components.join(", "),
                    component_refs
                ));
                self.indent();
                
                self.temp_vars.insert(format!("{}_entities", node.id), "entity".to_string());
            }
            
            "component/get" => {
                let entity_var = self.get_input(&node.id, "entity")
                    .unwrap_or_else(|| "entity".to_string());
                let component_type = node.data.get("componentType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Transform");
                
                let output_var = self.gen_var("component");
                self.write_line(&format!(
                    "if let Some({}) = world.get_component::<{}>({}) {{",
                    output_var, component_type, entity_var
                ));
                self.indent();
                
                self.temp_vars.insert(node.id.clone(), output_var);
            }
            
            "component/set" => {
                let entity_var = self.get_input(&node.id, "entity")
                    .unwrap_or_else(|| "entity".to_string());
                let component_var = self.get_input(&node.id, "component")
                    .ok_or_else(|| CompilerError::InvalidConnection("Missing component input".to_string()))?;
                
                self.write_line(&format!(
                    "world.add_component({}, {});",
                    entity_var, component_var
                ));
            }
            
            "transform/translate" => {
                let transform_var = self.get_input(&node.id, "transform")
                    .ok_or_else(|| CompilerError::InvalidConnection("Missing transform input".to_string()))?;
                let delta_var = self.get_input(&node.id, "delta")
                    .unwrap_or_else(|| "Vec3::ZERO".to_string());
                
                self.write_line(&format!(
                    "{}.position += {};",
                    transform_var, delta_var
                ));
            }
            
            "math/add" => {
                let a = self.get_input(&node.id, "a").unwrap_or_else(|| "0.0".to_string());
                let b = self.get_input(&node.id, "b").unwrap_or_else(|| "0.0".to_string());
                let output_var = self.gen_var("sum");
                
                self.write_line(&format!("let {} = {} + {};", output_var, a, b));
                self.temp_vars.insert(node.id.clone(), output_var);
            }
            
            "math/multiply" => {
                let a = self.get_input(&node.id, "a").unwrap_or_else(|| "1.0".to_string());
                let b = self.get_input(&node.id, "b").unwrap_or_else(|| "1.0".to_string());
                let output_var = self.gen_var("product");
                
                self.write_line(&format!("let {} = {} * {};", output_var, a, b));
                self.temp_vars.insert(node.id.clone(), output_var);
            }
            
            "flow/if" => {
                let condition = self.get_input(&node.id, "condition")
                    .unwrap_or_else(|| "false".to_string());
                
                self.write_line(&format!("if {} {{", condition));
                self.indent();
                // The then/else branches would be handled by connected nodes
            }
            
            "flow/foreach" => {
                let array = self.get_input(&node.id, "array")
                    .ok_or_else(|| CompilerError::InvalidConnection("Missing array input".to_string()))?;
                let item_var = self.gen_var("item");
                
                self.write_line(&format!("for {} in {}.iter() {{", item_var, array));
                self.indent();
                
                self.temp_vars.insert(format!("{}_item", node.id), item_var);
            }
            
            "action/spawn" => {
                let position = self.get_input(&node.id, "position")
                    .unwrap_or_else(|| "Vec3::ZERO".to_string());
                let prefab = node.data.get("prefab")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                
                self.write_line("let new_entity = world.create_entity();");
                self.write_line(&format!(
                    "world.add_component(new_entity, Transform::from_position({}));",
                    position
                ));
                
                // Add prefab-specific components
                self.write_line(&format!("// TODO: Load prefab '{}'", prefab));
            }
            
            "action/destroy" => {
                let entity = self.get_input(&node.id, "entity")
                    .unwrap_or_else(|| "entity".to_string());
                
                self.write_line(&format!("world.destroy_entity({});", entity));
            }
            
            _ => {
                return Err(CompilerError::UnknownNode(node.get_type().to_string()));
            }
        }
        
        Ok(())
    }
    
    fn topological_sort(
        &self,
        nodes: &[VisualScriptNode],
        connections: &[crate::VisualScriptConnection]
    ) -> Result<Vec<&VisualScriptNode>, CompilerError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        
        let node_map: HashMap<_, _> = nodes.iter()
            .map(|n| (n.id.as_str(), n))
            .collect();
        
        // Build adjacency list
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in nodes {
            graph.insert(&node.id, Vec::new());
        }
        
        for conn in connections {
            if let Some(neighbors) = graph.get_mut(conn.source.as_str()) {
                neighbors.push(&conn.target);
            }
        }
        
        fn visit<'a>(
            node_id: &str,
            graph: &HashMap<&str, Vec<&str>>,
            node_map: &HashMap<&str, &'a VisualScriptNode>,
            visited: &mut HashSet<String>,
            temp_visited: &mut HashSet<String>,
            sorted: &mut Vec<&'a VisualScriptNode>
        ) -> Result<(), CompilerError> {
            if temp_visited.contains(node_id) {
                return Err(CompilerError::InvalidConnection("Cycle detected".to_string()));
            }
            
            if !visited.contains(node_id) {
                temp_visited.insert(node_id.to_string());
                
                if let Some(neighbors) = graph.get(node_id) {
                    for &neighbor in neighbors {
                        visit(neighbor, graph, node_map, visited, temp_visited, sorted)?;
                    }
                }
                
                temp_visited.remove(node_id);
                visited.insert(node_id.to_string());
                
                if let Some(node) = node_map.get(node_id) {
                    sorted.push(node);
                }
            }
            
            Ok(())
        }
        
        for node in nodes {
            if !visited.contains(&node.id) {
                visit(&node.id, &graph, &node_map, &mut visited, &mut temp_visited, &mut sorted)?;
            }
        }
        
        sorted.reverse();
        Ok(sorted)
    }
    
    fn get_input(&self, node_id: &str, input_name: &str) -> Option<String> {
        self.temp_vars.get(&format!("{}_{}", node_id, input_name)).cloned()
    }
    
    fn gen_var(&mut self, prefix: &str) -> String {
        let var = format!("{}_{}", prefix, self.var_counter);
        self.var_counter += 1;
        var
    }
    
    fn write_line(&mut self, line: &str) {
        let indent = "    ".repeat(self.indent_level);
        self.code.push(format!("{}{}", indent, line));
    }
    
    fn indent(&mut self) {
        self.indent_level += 1;
    }
    
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
    
    fn to_rust_name(&self, name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_')
            .collect::<String>()
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

// Export functionality
pub use builder::{GameCompiler, BuildTarget, BuildResult};