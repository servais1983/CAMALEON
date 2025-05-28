#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_report_generation() {
        // Create a temporary directory for test output
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_report.html");
        
        // Create sample detection results
        let detections = vec![
            formats::DetectionResult {
                detection_type: "ransomware_indicator".to_string(),
                severity: 9,
                location: "row:5,col:3".to_string(),
                details: {
                    let mut map = HashMap::new();
                    map.insert("matched_text".to_string(), "suspicious ransomware activity".to_string());
                    map.insert("column".to_string(), "3".to_string());
                    map
                },
                timestamp: chrono::Utc::now(),
            },
            formats::DetectionResult {
                detection_type: "phishing_indicator".to_string(),
                severity: 7,
                location: "row:12,col:2".to_string(),
                details: {
                    let mut map = HashMap::new();
                    map.insert("matched_text".to_string(), "phishing attempt detected".to_string());
                    map.insert("column".to_string(), "2".to_string());
                    map
                },
                timestamp: chrono::Utc::now(),
            },
            formats::DetectionResult {
                detection_type: "suspicious_activity".to_string(),
                severity: 4,
                location: "row:18,col:5".to_string(),
                details: {
                    let mut map = HashMap::new();
                    map.insert("matched_text".to_string(), "unusual login pattern".to_string());
                    map.insert("column".to_string(), "5".to_string());
                    map
                },
                timestamp: chrono::Utc::now(),
            },
        ];
        
        // Create report generator
        let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");
        let report_generator = ReportGenerator::new(
            template_dir.to_str().unwrap(),
            temp_dir.path().to_str().unwrap(),
        ).unwrap();
        
        // Generate report
        let result = report_generator.generate_report(&detections, "test_report.html");
        
        // Verify report was generated successfully
        assert!(result.is_ok());
        assert!(output_path.exists());
        
        // Verify report content
        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("ransomware_indicator"));
        assert!(content.contains("phishing_indicator"));
        assert!(content.contains("suspicious_activity"));
    }
}
