use chame_core::events::{Event, EventType};
use handlebars::Handlebars;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur in the Reports module
#[derive(Error, Debug)]
pub enum ReportsError {
    #[error("Template error: {0}")]
    Template(#[from] handlebars::TemplateError),
    
    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Report generation service
pub struct ReportGenerator {
    /// Handlebars template engine
    handlebars: Handlebars<'static>,
    
    /// Template directory
    template_dir: String,
    
    /// Output directory
    output_dir: String,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(template_dir: &str, output_dir: &str) -> Result<Self, ReportsError> {
        let mut handlebars = Handlebars::new();
        
        // Register templates
        let template_path = Path::new(template_dir).join("report_template.html");
        handlebars.register_template_file("report", template_path)?;
        
        // Create output directory if it doesn't exist
        if !Path::new(output_dir).exists() {
            fs::create_dir_all(output_dir)?;
        }
        
        Ok(Self {
            handlebars,
            template_dir: template_dir.to_string(),
            output_dir: output_dir.to_string(),
        })
    }
    
    /// Generate a report from detections
    pub fn generate_report<P: AsRef<Path>>(
        &self,
        detections: &[formats::DetectionResult],
        output_file: P,
    ) -> Result<(), ReportsError> {
        // Calculate statistics
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        
        // Group detections by type
        let mut threat_types = HashMap::new();
        
        // Prepare detection data for template
        let detection_data: Vec<serde_json::Value> = detections
            .iter()
            .map(|d| {
                // Count by severity
                match d.severity {
                    8..=10 => high_count += 1,
                    5..=7 => medium_count += 1,
                    _ => low_count += 1,
                }
                
                // Group by type
                let entry = threat_types.entry(d.detection_type.clone()).or_insert_with(|| {
                    json!({
                        "type": d.detection_type,
                        "count": 0,
                        "total_severity": 0,
                    })
                });
                
                let mut entry_obj = entry.as_object_mut().unwrap();
                entry_obj["count"] = json!(entry_obj["count"].as_i64().unwrap() + 1);
                entry_obj["total_severity"] = json!(entry_obj["total_severity"].as_i64().unwrap() + d.severity as i64);
                
                // Create detection entry
                let severity_class = match d.severity {
                    8..=10 => "severity-high",
                    5..=7 => "severity-medium",
                    _ => "severity-low",
                };
                
                let severity_text = match d.severity {
                    8..=10 => "Critique",
                    5..=7 => "Moyenne",
                    _ => "Faible",
                };
                
                json!({
                    "title": format!("Détection: {}", d.detection_type),
                    "type": d.detection_type,
                    "severity": d.severity,
                    "severity_class": severity_class,
                    "severity_text": severity_text,
                    "location": d.location,
                    "timestamp": d.timestamp.to_rfc3339(),
                    "description": format!("Une activité suspecte de type {} a été détectée. Niveau de sévérité: {}.", 
                                          d.detection_type, d.severity),
                    "source": d.details.get("matched_text").cloned().unwrap_or_default(),
                })
            })
            .collect();
        
        // Calculate threat stats
        let threat_stats: Vec<serde_json::Value> = threat_types
            .iter()
            .map(|(_, value)| {
                let obj = value.as_object().unwrap();
                let count = obj["count"].as_i64().unwrap();
                let total_severity = obj["total_severity"].as_i64().unwrap();
                let avg_severity = if count > 0 {
                    (total_severity as f64 / count as f64).round()
                } else {
                    0.0
                };
                
                let status_class = match avg_severity as u8 {
                    8..=10 => "badge-danger",
                    5..=7 => "badge-warning",
                    _ => "badge-success",
                };
                
                let status = match avg_severity as u8 {
                    8..=10 => "Critique",
                    5..=7 => "Attention",
                    _ => "Normal",
                };
                
                json!({
                    "type": obj["type"],
                    "count": count,
                    "avg_severity": avg_severity,
                    "status": status,
                    "status_class": status_class,
                })
            })
            .collect();
        
        // Generate recommendations based on detections
        let mut recommendations = Vec::new();
        
        if high_count > 0 {
            recommendations.push("Effectuer une analyse forensique approfondie du système pour identifier les compromissions potentielles.".to_string());
            recommendations.push("Isoler immédiatement les systèmes affectés du réseau.".to_string());
        }
        
        if detections.iter().any(|d| d.detection_type.contains("ransomware")) {
            recommendations.push("Vérifier l'intégrité des sauvegardes et préparer un plan de restauration.".to_string());
            recommendations.push("Rechercher des indicateurs de compromission liés aux ransomwares sur tous les systèmes connectés.".to_string());
        }
        
        if detections.iter().any(|d| d.detection_type.contains("phishing")) {
            recommendations.push("Organiser une formation de sensibilisation à la sécurité pour tous les utilisateurs.".to_string());
            recommendations.push("Renforcer les filtres anti-phishing sur les passerelles de messagerie.".to_string());
        }
        
        // Add general recommendations
        recommendations.push("Mettre à jour tous les systèmes avec les derniers correctifs de sécurité.".to_string());
        recommendations.push("Renforcer les politiques de mot de passe et activer l'authentification à deux facteurs.".to_string());
        
        // Calculate overall score
        let total_count = high_count + medium_count + low_count;
        let weighted_score = if total_count > 0 {
            let score = 100 - ((high_count * 10 + medium_count * 5 + low_count * 2) as f64 / total_count as f64).round() as u8;
            score.min(100).max(0)
        } else {
            100
        };
        
        let score_class = match weighted_score {
            0..=40 => "score-high",
            41..=70 => "score-medium",
            _ => "score-low",
        };
        
        let summary_text = match weighted_score {
            0..=40 => "L'analyse a révélé des problèmes de sécurité critiques qui nécessitent une attention immédiate.",
            41..=70 => "L'analyse a identifié plusieurs problèmes de sécurité qui devraient être traités rapidement.",
            _ => "L'analyse n'a révélé que des problèmes mineurs ou aucun problème de sécurité significatif.",
        };
        
        // Prepare template data
        let data = json!({
            "date": chrono::Utc::now().format("%d/%m/%Y %H:%M").to_string(),
            "score": weighted_score,
            "score_class": score_class,
            "summary_text": summary_text,
            "high_count": high_count,
            "medium_count": medium_count,
            "low_count": low_count,
            "total_count": total_count,
            "detections": detection_data,
            "threat_stats": threat_stats,
            "recommendations": recommendations,
            "activities": detection_data,  // Reuse detection data for activities
        });
        
        // Render template
        let rendered = self.handlebars.render("report", &data)?;
        
        // Write to file
        let output_path = Path::new(&self.output_dir).join(output_file);
        fs::write(output_path, rendered)?;
        
        Ok(())
    }
}
