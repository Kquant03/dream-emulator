// Node type definitions for visual scripting

export interface NodeDefinition {
  type: string;
  category: string;
  inputs: PortDefinition[];
  outputs: PortDefinition[];
}

export interface PortDefinition {
  name: string;
  type: string;
}

// TODO: Define all node types
