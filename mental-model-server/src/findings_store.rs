//! Findings Store - SQLite-based storage for audit findings (ADR-0007)
//!
//! This module provides a dedicated storage backend for findings, separated
//! from the main mental model for improved performance and query capabilities.

use crate::model::{Finding, FindingContext, Severity};
use anyhow::Result;
use rusqlite::{Connection, params, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// SQL schema for the findings database
const SCHEMA_SQL: &str = r#"
-- Core findings table
CREATE TABLE IF NOT EXISTS findings (
    id TEXT PRIMARY KEY,
    viewpoint TEXT NOT NULL,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER,
    base_severity TEXT NOT NULL,
    adjusted_severity TEXT NOT NULL,
    rule_id TEXT,
    recommendation TEXT,
    context_json TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    CHECK (base_severity IN ('CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'INFO')),
    CHECK (adjusted_severity IN ('CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'INFO'))
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_findings_viewpoint ON findings(viewpoint);
CREATE INDEX IF NOT EXISTS idx_findings_file_path ON findings(file_path);
CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(adjusted_severity);
CREATE INDEX IF NOT EXISTS idx_findings_category ON findings(category);

-- Metadata table for store info
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT
);
"#;

/// Findings store backed by SQLite
pub struct FindingsStore {
    conn: Connection,
    path: PathBuf,
}

impl FindingsStore {
    /// Create or open a findings store at the given path
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;

        // Initialize schema
        conn.execute_batch(SCHEMA_SQL)?;

        // Set schema version if not exists
        conn.execute(
            "INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1')",
            [],
        )?;

        Ok(Self { conn, path })
    }

    /// Create an in-memory findings store (useful for testing)
    #[allow(dead_code)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA_SQL)?;
        conn.execute(
            "INSERT OR IGNORE INTO metadata (key, value) VALUES ('schema_version', '1')",
            [],
        )?;
        Ok(Self {
            conn,
            path: PathBuf::from(":memory:"),
        })
    }

    /// Get the path to the database file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Add a single finding
    pub fn add(&self, finding: &Finding) -> Result<()> {
        let context_json = finding.context.as_ref()
            .map(|c| serde_json::to_string(c))
            .transpose()?;

        self.conn.execute(
            "INSERT INTO findings (id, viewpoint, category, title, description,
             file_path, line_number, base_severity, adjusted_severity,
             rule_id, recommendation, context_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                finding.id,
                finding.viewpoint,
                finding.category,
                finding.title,
                finding.description,
                finding.file_path,
                finding.line_number,
                severity_to_string(&finding.base_severity),
                severity_to_string(&finding.adjusted_severity),
                finding.rule_id,
                finding.recommendation,
                context_json,
            ],
        )?;
        Ok(())
    }

    /// Add multiple findings in a single transaction
    pub fn add_batch(&self, findings: &[Finding]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;

        for finding in findings {
            let context_json = finding.context.as_ref()
                .map(|c| serde_json::to_string(c))
                .transpose()?;

            tx.execute(
                "INSERT INTO findings (id, viewpoint, category, title, description,
                 file_path, line_number, base_severity, adjusted_severity,
                 rule_id, recommendation, context_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    finding.id,
                    finding.viewpoint,
                    finding.category,
                    finding.title,
                    finding.description,
                    finding.file_path,
                    finding.line_number,
                    severity_to_string(&finding.base_severity),
                    severity_to_string(&finding.adjusted_severity),
                    finding.rule_id,
                    finding.recommendation,
                    context_json,
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get all findings
    pub fn get_all(&self) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings ORDER BY created_at"
        )?;

        self.query_to_findings(&mut stmt, [])
    }

    /// Get findings by file path (indexed query)
    pub fn get_by_file(&self, file_path: &str) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings WHERE file_path = ?1 ORDER BY created_at"
        )?;

        self.query_to_findings(&mut stmt, [file_path])
    }

    /// Get findings by severity (indexed query)
    pub fn get_by_severity(&self, severity: &Severity) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings WHERE adjusted_severity = ?1 ORDER BY created_at"
        )?;

        self.query_to_findings(&mut stmt, [severity_to_string(severity)])
    }

    /// Get findings by viewpoint (indexed query)
    pub fn get_by_viewpoint(&self, viewpoint: &str) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings WHERE viewpoint = ?1 ORDER BY created_at"
        )?;

        self.query_to_findings(&mut stmt, [viewpoint])
    }

    /// Get findings by category (indexed query)
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings WHERE category = ?1 ORDER BY created_at"
        )?;

        self.query_to_findings(&mut stmt, [category])
    }

    /// Get total finding count
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM findings",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Get counts grouped by severity
    pub fn counts_by_severity(&self) -> Result<HashMap<String, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT adjusted_severity, COUNT(*) FROM findings GROUP BY adjusted_severity"
        )?;

        let mut counts = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (severity, count) = row?;
            counts.insert(severity, count as usize);
        }

        Ok(counts)
    }

    /// Get counts grouped by category
    pub fn counts_by_category(&self) -> Result<HashMap<String, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, COUNT(*) FROM findings GROUP BY category"
        )?;

        let mut counts = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (category, count) = row?;
            counts.insert(category, count as usize);
        }

        Ok(counts)
    }

    /// Get counts grouped by viewpoint
    pub fn counts_by_viewpoint(&self) -> Result<HashMap<String, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT viewpoint, COUNT(*) FROM findings GROUP BY viewpoint"
        )?;

        let mut counts = HashMap::new();
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (viewpoint, count) = row?;
            counts.insert(viewpoint, count as usize);
        }

        Ok(counts)
    }

    /// Export all findings to JSON file
    pub fn export_json(&self, output_path: impl AsRef<Path>) -> Result<usize> {
        let findings = self.get_all()?;
        let count = findings.len();
        let json = serde_json::to_string_pretty(&findings)?;
        fs::write(output_path, json)?;
        Ok(count)
    }

    /// Clear all findings (useful for re-initialization)
    pub fn clear(&self) -> Result<usize> {
        let count = self.count()?;
        self.conn.execute("DELETE FROM findings", [])?;
        Ok(count)
    }

    /// Get a specific finding by ID
    pub fn get_by_id(&self, id: &str) -> Result<Option<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, viewpoint, category, title, description, file_path,
             line_number, base_severity, adjusted_severity, rule_id,
             recommendation, context_json
             FROM findings WHERE id = ?1"
        )?;

        stmt.query_row([id], |row| {
            Ok(row_to_finding(row))
        }).optional()?.transpose()
    }

    /// Get distinct viewpoints that have findings
    pub fn get_viewpoints_with_findings(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT viewpoint FROM findings ORDER BY viewpoint"
        )?;

        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut viewpoints = Vec::new();
        for row in rows {
            viewpoints.push(row?);
        }
        Ok(viewpoints)
    }

    /// Helper to convert query results to findings
    fn query_to_findings<P: rusqlite::Params>(
        &self,
        stmt: &mut rusqlite::Statement,
        params: P,
    ) -> Result<Vec<Finding>> {
        let rows = stmt.query_map(params, |row| Ok(row_to_finding(row)))?;

        let mut findings = Vec::new();
        for row in rows {
            findings.push(row??);
        }
        Ok(findings)
    }
}

/// Convert a database row to a Finding
fn row_to_finding(row: &rusqlite::Row) -> Result<Finding> {
    let context_json: Option<String> = row.get(11)?;
    let context: Option<FindingContext> = context_json
        .map(|json| serde_json::from_str(&json))
        .transpose()?;

    Ok(Finding {
        id: row.get(0)?,
        viewpoint: row.get(1)?,
        category: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        file_path: row.get(5)?,
        line_number: row.get(6)?,
        base_severity: string_to_severity(&row.get::<_, String>(7)?),
        adjusted_severity: string_to_severity(&row.get::<_, String>(8)?),
        rule_id: row.get(9)?,
        recommendation: row.get(10)?,
        context,
    })
}

/// Convert Severity enum to string for storage
fn severity_to_string(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical => "CRITICAL",
        Severity::High => "HIGH",
        Severity::Medium => "MEDIUM",
        Severity::Low => "LOW",
        Severity::Info => "INFO",
    }
}

/// Convert string to Severity enum
fn string_to_severity(s: &str) -> Severity {
    match s {
        "CRITICAL" => Severity::Critical,
        "HIGH" => Severity::High,
        "MEDIUM" => Severity::Medium,
        "LOW" => Severity::Low,
        _ => Severity::Info,
    }
}

/// Summary of findings for reporting
#[derive(Debug, Clone, serde::Serialize)]
pub struct FindingsSummary {
    pub total: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub by_viewpoint: HashMap<String, usize>,
}

impl FindingsStore {
    /// Get a complete summary of findings
    pub fn get_summary(&self) -> Result<FindingsSummary> {
        Ok(FindingsSummary {
            total: self.count()?,
            by_severity: self.counts_by_severity()?,
            by_category: self.counts_by_category()?,
            by_viewpoint: self.counts_by_viewpoint()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BoundedContextType;  // Used in create_test_finding

    fn create_test_finding(id: &str, viewpoint: &str, severity: Severity) -> Finding {
        Finding {
            id: id.to_string(),
            viewpoint: viewpoint.to_string(),
            category: "test_category".to_string(),
            title: "Test Finding".to_string(),
            description: "Test description".to_string(),
            file_path: "src/test.rs".to_string(),
            line_number: Some(42),
            base_severity: severity.clone(),
            adjusted_severity: severity,
            rule_id: Some("TEST-001".to_string()),
            recommendation: Some("Fix it".to_string()),
            context: Some(FindingContext {
                bounded_context: Some("TestContext".to_string()),
                bounded_context_type: Some(BoundedContextType::Core),
                layer: Some("domain".to_string()),
                is_hotspot: false,
                hotspot_score: None,
            }),
        }
    }

    #[test]
    fn test_create_store() {
        let store = FindingsStore::in_memory().unwrap();
        assert_eq!(store.count().unwrap(), 0);
    }

    #[test]
    fn test_add_and_get_finding() {
        let store = FindingsStore::in_memory().unwrap();
        let finding = create_test_finding("F-001", "VP-Q01", Severity::High);

        store.add(&finding).unwrap();

        assert_eq!(store.count().unwrap(), 1);

        let retrieved = store.get_by_id("F-001").unwrap().unwrap();
        assert_eq!(retrieved.id, "F-001");
        assert_eq!(retrieved.viewpoint, "VP-Q01");
        assert_eq!(retrieved.adjusted_severity, Severity::High);
    }

    #[test]
    fn test_add_batch() {
        let store = FindingsStore::in_memory().unwrap();
        let findings = vec![
            create_test_finding("F-001", "VP-Q01", Severity::High),
            create_test_finding("F-002", "VP-Q01", Severity::Medium),
            create_test_finding("F-003", "VP-Q02", Severity::Low),
        ];

        let count = store.add_batch(&findings).unwrap();

        assert_eq!(count, 3);
        assert_eq!(store.count().unwrap(), 3);
    }

    #[test]
    fn test_get_by_severity() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-003", "VP-Q01", Severity::Low)).unwrap();

        let high_findings = store.get_by_severity(&Severity::High).unwrap();
        assert_eq!(high_findings.len(), 2);

        let low_findings = store.get_by_severity(&Severity::Low).unwrap();
        assert_eq!(low_findings.len(), 1);
    }

    #[test]
    fn test_get_by_viewpoint() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q01", Severity::Medium)).unwrap();
        store.add(&create_test_finding("F-003", "VP-Q02", Severity::Low)).unwrap();

        let vp_q01 = store.get_by_viewpoint("VP-Q01").unwrap();
        assert_eq!(vp_q01.len(), 2);

        let vp_q02 = store.get_by_viewpoint("VP-Q02").unwrap();
        assert_eq!(vp_q02.len(), 1);
    }

    #[test]
    fn test_counts_by_severity() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-003", "VP-Q01", Severity::Low)).unwrap();

        let counts = store.counts_by_severity().unwrap();
        assert_eq!(counts.get("HIGH"), Some(&2));
        assert_eq!(counts.get("LOW"), Some(&1));
    }

    #[test]
    fn test_get_summary() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q02", Severity::Medium)).unwrap();

        let summary = store.get_summary().unwrap();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.by_viewpoint.len(), 2);
    }

    #[test]
    fn test_clear() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q01", Severity::Medium)).unwrap();

        assert_eq!(store.count().unwrap(), 2);

        let cleared = store.clear().unwrap();
        assert_eq!(cleared, 2);
        assert_eq!(store.count().unwrap(), 0);
    }

    #[test]
    fn test_viewpoints_with_findings() {
        let store = FindingsStore::in_memory().unwrap();
        store.add(&create_test_finding("F-001", "VP-Q01", Severity::High)).unwrap();
        store.add(&create_test_finding("F-002", "VP-Q02", Severity::Medium)).unwrap();
        store.add(&create_test_finding("F-003", "VP-Q01", Severity::Low)).unwrap();

        let viewpoints = store.get_viewpoints_with_findings().unwrap();
        assert_eq!(viewpoints.len(), 2);
        assert!(viewpoints.contains(&"VP-Q01".to_string()));
        assert!(viewpoints.contains(&"VP-Q02".to_string()));
    }
}
