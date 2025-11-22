use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type NodeId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocpackGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub repository_name: Option<String>,
    pub total_files: usize,
    pub total_symbols: usize,
    pub languages: HashSet<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub location: Location,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Function(FunctionNode),
    Type(TypeNode),
    Trait(TraitNode),
    Module(ModuleNode),
    Constant(ConstantNode),
    File(FileNode),
    Cluster(ClusterNode),
    Package(PackageNode),
    Macro(MacroNode),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FunctionNode {
    pub name: String,
    pub signature: String,
    pub is_public: bool,
    pub is_async: bool,
    pub is_method: bool,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TypeNode {
    pub name: String,
    pub kind: TypeKind,
    pub is_public: bool,
    pub fields: Vec<Field>,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Struct,
    Class,
    Enum,
    Interface,
    Trait,
    Union,
    TypeAlias,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TraitNode {
    pub name: String,
    pub is_public: bool,
    pub methods: Vec<String>,
    pub implementors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModuleNode {
    pub name: String,
    pub path: String,
    pub is_public: bool,
    pub children: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ConstantNode {
    pub name: String,
    pub value_type: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FileNode {
    pub path: String,
    pub language: String,
    pub size_bytes: usize,
    pub line_count: usize,
    pub symbols: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub name: String,
    pub topic: Option<String>,
    pub members: Vec<NodeId>,
    pub keywords: Vec<String>,
    pub centroid: Option<Vec<f32>>,
}

impl PartialEq for ClusterNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.topic == other.topic && self.members == other.members
    }
}

impl Eq for ClusterNode {}

impl std::hash::Hash for ClusterNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.topic.hash(state);
        self.members.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PackageNode {
    pub name: String,
    pub version: Option<String>,
    pub modules: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MacroNode {
    pub name: String,
    pub is_public: bool,
    pub macro_type: MacroType,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MacroType {
    Declarative,
    Procedural,
    Derive,
    Attribute,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub field_type: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Location {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    pub complexity: Option<u32>,
    pub fan_in: usize,
    pub fan_out: usize,
    pub is_public_api: bool,
    pub docstring: Option<String>,
    pub tags: Vec<String>,
    pub source_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub kind: EdgeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    Calls,
    Imports,
    TypeReference,
    DataFlow,
    ModuleOwnership,
    TraitImplementation,
    Inheritance,
    MethodOf,
    DefinedIn,
    InferredType,
    TraitMethodCall,
    MethodDispatch,
    MacroExpansion,
    TraitProvides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub version: String,
    pub generator: String,
    pub source: String,
    pub generated_at: String,
    pub files_included: usize,
    pub total_size_bytes: usize,
    pub format: String,
    pub contents: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub symbol_summaries: HashMap<NodeId, SymbolDocumentation>,
    #[serde(default)]
    pub module_overviews: HashMap<String, ModuleOverview>,
    pub architecture_overview: ArchitectureOverview,
    pub total_tokens_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDocumentation {
    pub node_id: NodeId,
    pub purpose: String,
    pub explanation: String,
    pub complexity_notes: Option<String>,
    pub usage_hints: Option<String>,
    #[serde(default)]
    pub caller_references: Vec<String>,
    #[serde(default)]
    pub callee_references: Vec<String>,
    pub semantic_cluster: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleOverview {
    pub module_name: String,
    pub responsibilities: String,
    #[serde(default)]
    pub key_symbols: Vec<String>,
    pub interactions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureOverview {
    pub overview: String,
    pub system_behavior: String,
    pub data_flow: String,
    #[serde(default)]
    pub key_components: Vec<String>,
}

impl Node {
    pub fn name(&self) -> String {
        match &self.kind {
            NodeKind::Function(f) => f.name.clone(),
            NodeKind::Type(t) => t.name.clone(),
            NodeKind::Trait(t) => t.name.clone(),
            NodeKind::Module(m) => m.name.clone(),
            NodeKind::Constant(c) => c.name.clone(),
            NodeKind::File(f) => f.path.clone(),
            NodeKind::Cluster(c) => c.name.clone(),
            NodeKind::Package(p) => p.name.clone(),
            NodeKind::Macro(m) => m.name.clone(),
        }
    }

    pub fn is_public(&self) -> bool {
        match &self.kind {
            NodeKind::Function(f) => f.is_public,
            NodeKind::Type(t) => t.is_public,
            NodeKind::Trait(t) => t.is_public,
            NodeKind::Module(m) => m.is_public,
            NodeKind::Constant(c) => c.is_public,
            NodeKind::Macro(m) => m.is_public,
            NodeKind::File(_) | NodeKind::Cluster(_) | NodeKind::Package(_) => true,
        }
    }

    pub fn kind_str(&self) -> &str {
        match &self.kind {
            NodeKind::Function(_) => "function",
            NodeKind::Type(_) => "type",
            NodeKind::Trait(_) => "trait",
            NodeKind::Module(_) => "module",
            NodeKind::Constant(_) => "constant",
            NodeKind::File(_) => "file",
            NodeKind::Cluster(_) => "cluster",
            NodeKind::Package(_) => "package",
            NodeKind::Macro(_) => "macro",
        }
    }
}
