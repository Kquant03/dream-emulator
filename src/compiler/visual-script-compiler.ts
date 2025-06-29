// src/compiler/visual-script-compiler.ts
import { VisualScript, VisualScriptNode, VisualScriptConnection } from '../store';

export interface CompiledSystem {
  name: string;
  code: string;
  dependencies: string[];
}

export class VisualScriptCompiler {
  private indentLevel = 0;
  private output: string[] = [];
  private nodeOutputs = new Map<string, string>();

  compile(script: VisualScript): CompiledSystem {
    this.reset();
    
    // Analyze node graph
    const nodeMap = new Map(script.nodes.map(n => [n.id, n]));
    const connectionMap = this.buildConnectionMap(script.connections);
    const sortedNodes = this.topologicalSort(script.nodes, script.connections);
    
    // Generate system struct
    this.writeLine(`pub struct ${this.toRustName(script.name)}System {`);
    this.indent();
    this.writeLine('// System state');
    this.dedent();
    this.writeLine('}');
    this.writeLine('');
    
    // Generate system implementation
    this.writeLine(`impl System for ${this.toRustName(script.name)}System {`);
    this.indent();
    
    // Generate execute method
    this.writeLine('fn execute(&mut self, world: &mut World, physics: &mut PhysicsWorld, dt: f32) {');
    this.indent();
    
    // Compile each node in topological order
    for (const node of sortedNodes) {
      this.compileNode(node, connectionMap, nodeMap);
    }
    
    this.dedent();
    this.writeLine('}');
    this.dedent();
    this.writeLine('}');
    
    return {
      name: script.name,
      code: this.output.join('\n'),
      dependencies: this.extractDependencies(script.nodes),
    };
  }

  private compileNode(
    node: VisualScriptNode,
    connections: Map<string, VisualScriptConnection[]>,
    nodeMap: Map<string, VisualScriptNode>
  ) {
    this.writeLine(`// Node: ${node.data.label || node.type}`);
    
    switch (node.type) {
      case 'event/update':
        this.compileUpdateEvent(node);
        break;
      
      case 'event/collision':
        this.compileCollisionEvent(node);
        break;
      
      case 'query/get_entities':
        this.compileGetEntities(node);
        break;
      
      case 'component/get':
        this.compileGetComponent(node, connections);
        break;
      
      case 'component/set':
        this.compileSetComponent(node, connections);
        break;
      
      case 'math/add':
        this.compileMathOperation(node, connections, '+');
        break;
      
      case 'math/multiply':
        this.compileMathOperation(node, connections, '*');
        break;
      
      case 'flow/if':
        this.compileIfStatement(node, connections, nodeMap);
        break;
      
      case 'flow/foreach':
        this.compileForEach(node, connections, nodeMap);
        break;
      
      default:
        this.writeLine(`// TODO: Implement node type ${node.type}`);
    }
    
    this.writeLine('');
  }

  private compileUpdateEvent(node: VisualScriptNode) {
    // Update event doesn't generate code itself, it's the entry point
    this.nodeOutputs.set(node.id, 'dt');
  }

  private compileCollisionEvent(node: VisualScriptNode) {
    this.writeLine('for (entity_a, entity_b) in physics.get_collision_pairs() {');
    this.indent();
    this.nodeOutputs.set(node.id + '_a', 'entity_a');
    this.nodeOutputs.set(node.id + '_b', 'entity_b');
  }

  private compileGetEntities(node: VisualScriptNode) {
    const components = node.data.components as string[] || ['Transform'];
    const queryVar = `query_${this.sanitizeId(node.id)}`;
    
    const componentTypes = components.map(c => `&${c}`).join(', ');
    this.writeLine(`let ${queryVar} = world.query::<(${componentTypes})>();`);
    
    this.nodeOutputs.set(node.id, queryVar);
  }

  private compileGetComponent(node: VisualScriptNode, connections: Map<string, VisualScriptConnection[]>) {
    const entityInput = this.getInputValue(node.id, 'entity', connections);
    const componentType = node.data.componentType || 'Transform';
    const outputVar = `comp_${this.sanitizeId(node.id)}`;
    
    this.writeLine(`let ${outputVar} = world.get_component::<${componentType}>(${entityInput})?;`);
    this.nodeOutputs.set(node.id, outputVar);
  }

  private compileSetComponent(node: VisualScriptNode, connections: Map<string, VisualScriptConnection[]>) {
    const entityInput = this.getInputValue(node.id, 'entity', connections);
    const componentInput = this.getInputValue(node.id, 'component', connections);
    const componentType = node.data.componentType || 'Transform';
    
    this.writeLine(`world.set_component(${entityInput}, ${componentInput});`);
  }

  private compileMathOperation(
    node: VisualScriptNode,
    connections: Map<string, VisualScriptConnection[]>,
    operator: string
  ) {
    const leftInput = this.getInputValue(node.id, 'a', connections) || '0.0';
    const rightInput = this.getInputValue(node.id, 'b', connections) || '0.0';
    const outputVar = `result_${this.sanitizeId(node.id)}`;
    
    this.writeLine(`let ${outputVar} = ${leftInput} ${operator} ${rightInput};`);
    this.nodeOutputs.set(node.id, outputVar);
  }

  private compileIfStatement(
    node: VisualScriptNode,
    connections: Map<string, VisualScriptConnection[]>,
    nodeMap: Map<string, VisualScriptNode>
  ) {
    const conditionInput = this.getInputValue(node.id, 'condition', connections) || 'false';
    
    this.writeLine(`if ${conditionInput} {`);
    this.indent();
    
    // Compile nodes connected to 'then' output
    const thenConnections = connections.get(node.id + '_then') || [];
    for (const conn of thenConnections) {
      const targetNode = nodeMap.get(conn.target);
      if (targetNode) {
        this.compileNode(targetNode, connections, nodeMap);
      }
    }
    
    this.dedent();
    
    // Check for else branch
    const elseConnections = connections.get(node.id + '_else') || [];
    if (elseConnections.length > 0) {
      this.writeLine('} else {');
      this.indent();
      
      for (const conn of elseConnections) {
        const targetNode = nodeMap.get(conn.target);
        if (targetNode) {
          this.compileNode(targetNode, connections, nodeMap);
        }
      }
      
      this.dedent();
    }
    
    this.writeLine('}');
  }

  private compileForEach(
    node: VisualScriptNode,
    connections: Map<string, VisualScriptConnection[]>,
    nodeMap: Map<string, VisualScriptNode>
  ) {
    const arrayInput = this.getInputValue(node.id, 'array', connections);
    const itemVar = `item_${this.sanitizeId(node.id)}`;
    
    this.writeLine(`for ${itemVar} in ${arrayInput}.iter() {`);
    this.indent();
    
    this.nodeOutputs.set(node.id + '_item', itemVar);
    
    // Compile loop body
    const bodyConnections = connections.get(node.id + '_body') || [];
    for (const conn of bodyConnections) {
      const targetNode = nodeMap.get(conn.target);
      if (targetNode) {
        this.compileNode(targetNode, connections, nodeMap);
      }
    }
    
    this.dedent();
    this.writeLine('}');
  }

  private getInputValue(nodeId: string, inputName: string, connections: Map<string, VisualScriptConnection[]>): string | null {
    const inputKey = `${nodeId}_${inputName}`;
    
    // Find connection to this input
    for (const [sourceKey, conns] of connections.entries()) {
      for (const conn of conns) {
        if (conn.target === nodeId && conn.targetHandle === inputName) {
          // Get the output value from the source node
          return this.nodeOutputs.get(conn.source) || null;
        }
      }
    }
    
    return null;
  }

  private buildConnectionMap(connections: VisualScriptConnection[]): Map<string, VisualScriptConnection[]> {
    const map = new Map<string, VisualScriptConnection[]>();
    
    for (const conn of connections) {
      const key = `${conn.source}_${conn.sourceHandle}`;
      if (!map.has(key)) {
        map.set(key, []);
      }
      map.get(key)!.push(conn);
    }
    
    return map;
  }

  private topologicalSort(nodes: VisualScriptNode[], connections: VisualScriptConnection[]): VisualScriptNode[] {
    // Build adjacency list
    const graph = new Map<string, string[]>();
    const inDegree = new Map<string, number>();
    
    for (const node of nodes) {
      graph.set(node.id, []);
      inDegree.set(node.id, 0);
    }
    
    for (const conn of connections) {
      graph.get(conn.source)!.push(conn.target);
      inDegree.set(conn.target, (inDegree.get(conn.target) || 0) + 1);
    }
    
    // Kahn's algorithm
    const queue: string[] = [];
    const sorted: VisualScriptNode[] = [];
    const nodeMap = new Map(nodes.map(n => [n.id, n]));
    
    // Find nodes with no inputs
    for (const [nodeId, degree] of inDegree.entries()) {
      if (degree === 0) {
        queue.push(nodeId);
      }
    }
    
    while (queue.length > 0) {
      const nodeId = queue.shift()!;
      sorted.push(nodeMap.get(nodeId)!);
      
      for (const neighbor of graph.get(nodeId)!) {
        const degree = inDegree.get(neighbor)! - 1;
        inDegree.set(neighbor, degree);
        
        if (degree === 0) {
          queue.push(neighbor);
        }
      }
    }
    
    return sorted;
  }

  private extractDependencies(nodes: VisualScriptNode[]): string[] {
    const deps = new Set<string>();
    
    for (const node of nodes) {
      // Extract component types
      if (node.data.componentType) {
        deps.add(node.data.componentType as string);
      }
      
      // Extract other dependencies based on node type
      switch (node.type) {
        case 'physics/raycast':
          deps.add('physics');
          break;
        case 'audio/play':
          deps.add('audio');
          break;
        // Add more as needed
      }
    }
    
    return Array.from(deps);
  }

  private reset() {
    this.indentLevel = 0;
    this.output = [];
    this.nodeOutputs.clear();
  }

  private writeLine(line: string) {
    const indent = '    '.repeat(this.indentLevel);
    this.output.push(indent + line);
  }

  private indent() {
    this.indentLevel++;
  }

  private dedent() {
    this.indentLevel--;
  }

  private toRustName(name: string): string {
    return name
      .split(/[\s-_]+/)
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join('');
  }

  private sanitizeId(id: string): string {
    return id.replace(/[^a-zA-Z0-9_]/g, '_');
  }
}

// Example usage in the editor
export async function compileAndPreview(script: VisualScript): Promise<void> {
  const compiler = new VisualScriptCompiler();
  const compiled = compiler.compile(script);
  
  // Send to Rust for compilation
  const { invoke } = await import('@tauri-apps/api/tauri');
  const result = await invoke('compile_visual_script', {
    scriptJson: JSON.stringify(script)
  });
  
  console.log('Compiled Rust code:', result);
}