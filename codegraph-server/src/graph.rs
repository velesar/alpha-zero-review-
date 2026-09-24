//! Codegraph data structures
//!
//! Provides data structures for semantic code analysis using SCIP indices.

use crate::scip;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// A symbol in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub file: String,
    pub range: Range,
    pub documentation: Option<String>,
}

/// Kind of symbol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Class,
    Function,
    Method,
    Variable,
    Constant,
    Module,
    Interface,
    Type,
    Property,
    Unknown,
}

/// A range in source code
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Range {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// A reference to a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub symbol_id: String,
    pub file: String,
    pub line: u32,
    pub role: ReferenceRole,
}

/// Role of a reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRole {
    Definition,
    Reference,
    Call,
    Import,
    TypeReference,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize)]
pub struct Impact {
    pub direct_references: usize,
    pub affected_files: usize,
    pub files: Vec<String>,
}

/// Module dependencies result
#[derive(Debug, Clone, Serialize)]
pub struct ModuleDeps {
    pub module: String,
    pub files: Vec<String>,
    pub symbols_count: usize,
    /// Files outside the module whose symbols this module references (file -> reference count)
    pub depends_on: HashMap<String, usize>,
    /// Files outside the module that reference this module's symbols (file -> reference count)
    pub dependents: HashMap<String, usize>,
}

/// Hotspot symbol
#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    pub symbol_id: String,
    pub name: String,
    pub file: String,
    pub kind: SymbolKind,
    pub caller_count: usize,
}

/// The main codegraph structure
#[derive(Debug, Default)]
pub struct Codegraph {
    /// All symbols indexed by ID
    pub symbols: HashMap<String, Symbol>,
    /// Symbol definitions: symbol_id -> [references where defined]
    pub definitions: HashMap<String, Vec<Reference>>,
    /// Symbol references: symbol_id -> [references where used]
    pub references: HashMap<String, Vec<Reference>>,
    /// Symbols by file: file_path -> [symbol_ids]
    pub file_symbols: HashMap<String, Vec<String>>,
    /// Definition bodies (SCIP enclosing_range): symbol_id -> range
    pub bodies: HashMap<String, Range>,
}

impl Codegraph {
    /// Create a new empty codegraph
    pub fn new() -> Self {
        Self::default()
    }

    /// Load codegraph from a SCIP index file
    pub fn load_from_scip(path: &Path) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        Self::parse_scip_index(&data)
    }

    /// Parse SCIP index data
    fn parse_scip_index(data: &[u8]) -> Result<Self, std::io::Error> {
        if data.is_empty() {
            return Ok(Self::new());
        }

        // Parse the protobuf-encoded SCIP index
        let index = scip::Index::decode(data).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse SCIP index: {}", e),
            )
        })?;

        let mut graph = Self::new();

        // Process each document in the index
        for doc in &index.documents {
            let file_path = doc.relative_path.clone();

            // Process symbol definitions in this document
            for sym_info in &doc.symbols {
                let symbol_id = sym_info.symbol.clone();
                if symbol_id.is_empty() {
                    continue;
                }

                // Extract symbol name from the SCIP symbol string
                let name = Self::extract_symbol_name(&symbol_id);
                let kind = Self::scip_kind_to_symbol_kind(sym_info.kind);

                // Get documentation from the symbol info
                let documentation = sym_info.documentation.first().cloned();

                let symbol = Symbol {
                    id: symbol_id.clone(),
                    kind: kind.clone(),
                    name,
                    file: file_path.clone(),
                    range: Range::default(), // Set from the definition occurrence below
                    documentation,
                };

                graph.symbols.insert(symbol_id.clone(), symbol);
                graph
                    .file_symbols
                    .entry(file_path.clone())
                    .or_default()
                    .push(symbol_id);
            }

            // Process occurrences (references and definitions)
            for occ in &doc.occurrences {
                let symbol_id = occ.symbol.clone();
                if symbol_id.is_empty() {
                    continue;
                }

                let range = Self::parse_scip_range(&occ.range).unwrap_or_default();
                let line = range.start_line;

                // Determine if this is a definition or reference based on symbol_roles
                let is_definition = occ.symbol_roles & (scip::SymbolRole::Definition as i32) != 0;

                if is_definition {
                    if let Some(symbol) = graph.symbols.get_mut(&symbol_id) {
                        if symbol.file == file_path {
                            symbol.range = range.clone();
                        }
                    }
                    if let Some(body) = Self::parse_scip_range(&occ.enclosing_range) {
                        graph.bodies.insert(symbol_id.clone(), body);
                    }
                }

                let reference = Reference {
                    symbol_id: symbol_id.clone(),
                    file: file_path.clone(),
                    line,
                    role: if is_definition {
                        ReferenceRole::Definition
                    } else {
                        ReferenceRole::Reference
                    },
                };

                if is_definition {
                    graph
                        .definitions
                        .entry(symbol_id)
                        .or_default()
                        .push(reference);
                } else {
                    graph
                        .references
                        .entry(symbol_id)
                        .or_default()
                        .push(reference);
                }
            }
        }

        // Also process external symbols (symbols defined in other packages)
        for sym_info in &index.external_symbols {
            let symbol_id = sym_info.symbol.clone();
            if symbol_id.is_empty() || graph.symbols.contains_key(&symbol_id) {
                continue;
            }

            let name = Self::extract_symbol_name(&symbol_id);
            let kind = Self::scip_kind_to_symbol_kind(sym_info.kind);
            let documentation = sym_info.documentation.first().cloned();

            let symbol = Symbol {
                id: symbol_id.clone(),
                kind,
                name,
                file: String::new(), // External symbol, no file
                range: Range::default(),
                documentation,
            };

            graph.symbols.insert(symbol_id, symbol);
        }

        tracing::info!(
            "Loaded SCIP index: {} symbols, {} files",
            graph.symbols.len(),
            graph.file_symbols.len()
        );

        Ok(graph)
    }

    /// Parse a SCIP range: [startLine, startChar, endLine, endChar] or
    /// [startLine, startChar, endChar] for single-line ranges.
    fn parse_scip_range(range: &[i32]) -> Option<Range> {
        let r = |i: usize| range[i].max(0) as u32;
        match range.len() {
            3 => Some(Range {
                start_line: r(0),
                start_column: r(1),
                end_line: r(0),
                end_column: r(2),
            }),
            4 => Some(Range {
                start_line: r(0),
                start_column: r(1),
                end_line: r(2),
                end_column: r(3),
            }),
            _ => None,
        }
    }

    /// Whether a SCIP symbol is local to a document (e.g. "local 3")
    fn is_local_symbol(symbol_id: &str) -> bool {
        symbol_id.starts_with("local ")
    }

    /// Whether a SCIP symbol names a function/method (`().`) or type (`#`),
    /// i.e. something whose definition bounds the body of the previous one.
    fn is_item_symbol(symbol_id: &str) -> bool {
        symbol_id.ends_with("().") || symbol_id.ends_with('#')
    }

    /// Merge another graph into this one (e.g. indexes for several languages).
    /// SCIP symbol IDs carry the indexer scheme, so they do not collide.
    pub fn merge(&mut self, other: Codegraph) {
        self.symbols.extend(other.symbols);
        for (id, refs) in other.definitions {
            self.definitions.entry(id).or_default().extend(refs);
        }
        for (id, refs) in other.references {
            self.references.entry(id).or_default().extend(refs);
        }
        for (file, ids) in other.file_symbols {
            self.file_symbols.entry(file).or_default().extend(ids);
        }
        self.bodies.extend(other.bodies);
    }

    /// Extract a readable name from a SCIP symbol string
    fn extract_symbol_name(symbol_id: &str) -> String {
        // SCIP symbol format: "scheme package descriptor"
        // Example: "scip-typescript npm @types/node 18.0.0 path/`join`()."
        // We want to extract the last meaningful part

        symbol_id
            .split('/')
            .next_back()
            .and_then(|s| s.split('`').nth(1))
            .or_else(|| symbol_id.split('/').next_back())
            .unwrap_or(symbol_id)
            .trim_end_matches(['(', ')', '.', '#'])
            .to_string()
    }

    /// Convert SCIP SymbolKind to our SymbolKind
    fn scip_kind_to_symbol_kind(kind: i32) -> SymbolKind {
        use scip::symbol_information::Kind;
        match Kind::try_from(kind) {
            Ok(Kind::Class) | Ok(Kind::Object) => SymbolKind::Class,
            Ok(Kind::Function) => SymbolKind::Function,
            Ok(Kind::Method) | Ok(Kind::Constructor) => SymbolKind::Method,
            Ok(Kind::Variable) | Ok(Kind::Parameter) => SymbolKind::Variable,
            Ok(Kind::Constant) => SymbolKind::Constant,
            Ok(Kind::Module) | Ok(Kind::Namespace) | Ok(Kind::Package) => SymbolKind::Module,
            Ok(Kind::Interface) | Ok(Kind::Trait) => SymbolKind::Interface,
            Ok(Kind::Type) | Ok(Kind::TypeAlias) | Ok(Kind::TypeParameter) => SymbolKind::Type,
            Ok(Kind::Property) | Ok(Kind::Field) => SymbolKind::Property,
            _ => SymbolKind::Unknown,
        }
    }

    /// Load from a JSON representation (for testing/alternative format)
    pub fn load_from_json(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let graph: Codegraph = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(graph)
    }

    /// Add a symbol to the graph
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let id = symbol.id.clone();
        let file = symbol.file.clone();

        self.symbols.insert(id.clone(), symbol);
        self.file_symbols.entry(file).or_default().push(id);
    }

    /// Set the body range of a symbol's definition
    pub fn set_body(&mut self, symbol_id: &str, body: Range) {
        self.bodies.insert(symbol_id.to_string(), body);
    }

    /// Add a reference
    pub fn add_reference(&mut self, reference: Reference) {
        let symbol_id = reference.symbol_id.clone();

        if reference.role == ReferenceRole::Definition {
            self.definitions
                .entry(symbol_id)
                .or_default()
                .push(reference);
        } else {
            self.references
                .entry(symbol_id)
                .or_default()
                .push(reference);
        }
    }

    /// Get a symbol by ID
    pub fn get_symbol(&self, id: &str) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    /// Get all callers/references of a symbol
    pub fn get_callers(&self, symbol_id: &str) -> Vec<&Reference> {
        self.references
            .get(symbol_id)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Line span of a symbol's definition body.
    ///
    /// Uses the SCIP `enclosing_range` when the indexer emitted one. Otherwise
    /// approximates the body as running from the definition to the line before
    /// the next function/type definition in the same file (or end of file).
    pub fn body_lines(&self, symbol_id: &str) -> Option<(String, u32, u32)> {
        let (file, line) = match self.definitions.get(symbol_id).and_then(|d| d.first()) {
            Some(def) => (def.file.clone(), def.line),
            None => {
                let symbol = self.symbols.get(symbol_id).filter(|s| !s.file.is_empty())?;
                (symbol.file.clone(), symbol.range.start_line)
            }
        };

        if let Some(body) = self.bodies.get(symbol_id) {
            return Some((file, body.start_line, body.end_line));
        }

        let next_def = self
            .definitions
            .iter()
            .filter(|(id, _)| id.as_str() != symbol_id && Self::is_item_symbol(id))
            .flat_map(|(_, defs)| defs)
            .filter(|d| d.file == file && d.line > line)
            .map(|d| d.line)
            .min();

        Some((file, line, next_def.map(|l| l - 1).unwrap_or(u32::MAX)))
    }

    /// Get callees from a symbol: references to other symbols made within its
    /// definition body, sorted by line. Local symbols are excluded.
    pub fn get_callees(&self, symbol_id: &str) -> Vec<&Reference> {
        let Some((file, start, end)) = self.body_lines(symbol_id) else {
            return Vec::new();
        };

        let mut callees: Vec<&Reference> = self
            .references
            .iter()
            .filter(|(id, _)| id.as_str() != symbol_id && !Self::is_local_symbol(id))
            .flat_map(|(_, refs)| refs)
            .filter(|r| r.file == file && r.line >= start && r.line <= end)
            .collect();
        callees.sort_by_key(|r| r.line);
        callees
    }

    /// Files in which a symbol is defined. A symbol can have several
    /// definitions (e.g. a crate root shared by lib.rs, main.rs and tests).
    fn defining_files(&self, symbol_id: &str) -> Vec<&str> {
        let mut files: Vec<&str> = self
            .definitions
            .get(symbol_id)
            .map(|defs| defs.iter().map(|d| d.file.as_str()).collect())
            .unwrap_or_default();
        if files.is_empty() {
            if let Some(symbol) = self.symbols.get(symbol_id).filter(|s| !s.file.is_empty()) {
                files.push(symbol.file.as_str());
            }
        }
        files.dedup();
        files
    }

    /// Get impact analysis for a symbol
    pub fn get_impact(&self, symbol_id: &str) -> Impact {
        let refs = self.get_callers(symbol_id);
        let affected_files: HashSet<_> = refs.iter().map(|r| r.file.as_str()).collect();

        Impact {
            direct_references: refs.len(),
            affected_files: affected_files.len(),
            files: affected_files.into_iter().map(String::from).collect(),
        }
    }

    /// Get module dependencies.
    ///
    /// `module_path` is a file path or a directory prefix (e.g. "src/auth/").
    /// `depends_on` lists files outside the module whose symbols it references;
    /// `dependents` lists files outside the module that reference its symbols.
    pub fn get_module_deps(&self, module_path: &str) -> ModuleDeps {
        let dir_prefix = format!("{}/", module_path.trim_end_matches('/'));
        let in_module = |file: &str| file == module_path || file.starts_with(&dir_prefix);

        let mut files: Vec<String> = self
            .file_symbols
            .keys()
            .filter(|f| in_module(f))
            .cloned()
            .collect();
        files.sort();

        let symbols_count = files
            .iter()
            .filter_map(|f| self.file_symbols.get(f))
            .map(|ids| ids.len())
            .sum();

        let mut depends_on: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, usize> = HashMap::new();

        for (symbol_id, refs) in &self.references {
            if Self::is_local_symbol(symbol_id) {
                continue;
            }
            let def_files = self.defining_files(symbol_id);
            let Some(&def_file) = def_files.first() else {
                continue;
            };
            let def_inside = def_files.iter().any(|f| in_module(f));

            for reference in refs {
                let ref_inside = in_module(&reference.file);
                if ref_inside && !def_inside {
                    *depends_on.entry(def_file.to_string()).or_default() += 1;
                } else if !ref_inside && def_inside {
                    *dependents.entry(reference.file.clone()).or_default() += 1;
                }
            }
        }

        ModuleDeps {
            module: module_path.to_string(),
            files,
            symbols_count,
            depends_on,
            dependents,
        }
    }

    /// Get symbols in a file
    pub fn get_file_symbols(&self, file_path: &str) -> Vec<&Symbol> {
        self.file_symbols
            .get(file_path)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find symbols by name pattern
    pub fn find_symbol(&self, pattern: &str) -> Vec<&Symbol> {
        let pattern_lower = pattern.to_lowercase();
        self.symbols
            .values()
            .filter(|s| {
                s.name.to_lowercase().contains(&pattern_lower)
                    || s.id.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    /// Find hotspot symbols (symbols with many callers)
    pub fn find_hotspots(&self, min_callers: usize, path_filter: Option<&str>) -> Vec<Hotspot> {
        self.symbols
            .values()
            .filter(|s| {
                if let Some(filter) = path_filter {
                    s.file.contains(filter)
                } else {
                    true
                }
            })
            .filter_map(|s| {
                let caller_count = self.get_callers(&s.id).len();
                if caller_count >= min_callers {
                    Some(Hotspot {
                        symbol_id: s.id.clone(),
                        name: s.name.clone(),
                        file: s.file.clone(),
                        kind: s.kind.clone(),
                        caller_count,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get total symbol count
    pub fn symbols_count(&self) -> usize {
        self.symbols.len()
    }

    /// Get total file count
    pub fn files_count(&self) -> usize {
        self.file_symbols.len()
    }
}

// Allow serialization for Codegraph (for JSON format support)
impl Serialize for Codegraph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Codegraph", 5)?;
        state.serialize_field("symbols", &self.symbols)?;
        state.serialize_field("definitions", &self.definitions)?;
        state.serialize_field("references", &self.references)?;
        state.serialize_field("file_symbols", &self.file_symbols)?;
        state.serialize_field("bodies", &self.bodies)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Codegraph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CodegraphHelper {
            symbols: HashMap<String, Symbol>,
            definitions: HashMap<String, Vec<Reference>>,
            references: HashMap<String, Vec<Reference>>,
            file_symbols: HashMap<String, Vec<String>>,
            #[serde(default)]
            bodies: HashMap<String, Range>,
        }

        let helper = CodegraphHelper::deserialize(deserializer)?;
        Ok(Codegraph {
            symbols: helper.symbols,
            definitions: helper.definitions,
            references: helper.references,
            file_symbols: helper.file_symbols,
            bodies: helper.bodies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegraph_new() {
        let graph = Codegraph::new();
        assert_eq!(graph.symbols_count(), 0);
        assert_eq!(graph.files_count(), 0);
    }

    #[test]
    fn test_add_symbol() {
        let mut graph = Codegraph::new();

        let symbol = Symbol {
            id: "test#func".to_string(),
            kind: SymbolKind::Function,
            name: "test_func".to_string(),
            file: "src/main.py".to_string(),
            range: Range::default(),
            documentation: None,
        };

        graph.add_symbol(symbol);
        assert_eq!(graph.symbols_count(), 1);
        assert!(graph.get_symbol("test#func").is_some());
    }

    #[test]
    fn test_find_symbol() {
        let mut graph = Codegraph::new();

        graph.add_symbol(Symbol {
            id: "test#process_data".to_string(),
            kind: SymbolKind::Function,
            name: "process_data".to_string(),
            file: "src/utils.py".to_string(),
            range: Range::default(),
            documentation: None,
        });

        let found = graph.find_symbol("process");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "process_data");
    }
}
