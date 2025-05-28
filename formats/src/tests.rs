#[cfg(test)]
mod tests {
    use formats::{FileAnalyzer, FileFormat, FormatsError};
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_csv_analyzer() {
        // Create a temporary CSV file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,description,status").unwrap();
        writeln!(temp_file, "1,test1,normal activity,ok").unwrap();
        writeln!(temp_file, "2,test2,suspicious activity detected,warning").unwrap();
        writeln!(temp_file, "3,test3,ransomware indicators found,critical").unwrap();
        writeln!(temp_file, "4,test4,potential phishing attempt,warning").unwrap();
        
        // Create CSV analyzer
        let analyzer = super::CsvAnalyzer::new();
        
        // Analyze the file
        let results = analyzer.analyze(temp_file.path()).unwrap();
        
        // Verify results
        assert_eq!(results.len(), 3); // Should detect 3 issues: suspicious, ransomware, phishing
        
        // Check for ransomware detection
        let ransomware_detection = results.iter().find(|r| r.detection_type == "ransomware_indicator").unwrap();
        assert_eq!(ransomware_detection.severity, 9);
        assert!(ransomware_detection.location.contains("row:3"));
        
        // Check for phishing detection
        let phishing_detection = results.iter().find(|r| r.detection_type == "phishing_indicator").unwrap();
        assert_eq!(phishing_detection.severity, 7);
        assert!(phishing_detection.location.contains("row:4"));
        
        // Check for suspicious activity detection
        let suspicious_detection = results.iter().find(|r| r.detection_type == "suspicious_activity").unwrap();
        assert_eq!(suspicious_detection.severity, 5);
        assert!(suspicious_detection.location.contains("row:2"));
    }

    #[test]
    fn test_log_analyzer() {
        // Create a temporary log file with test data
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2025-05-28 05:30:00 INFO: System started normally").unwrap();
        writeln!(temp_file, "2025-05-28 05:35:12 WARNING: Failed login attempt from 192.168.1.100").unwrap();
        writeln!(temp_file, "2025-05-28 05:36:45 ERROR: Multiple authentication failures, possible brute force attempt").unwrap();
        writeln!(temp_file, "2025-05-28 05:40:22 CRITICAL: Suspicious activity detected, possible malware execution").unwrap();
        
        // Create Log analyzer
        let analyzer = super::LogAnalyzer::new();
        
        // Analyze the file
        let results = analyzer.analyze(temp_file.path()).unwrap();
        
        // Verify results
        assert!(results.len() >= 2); // Should detect at least 2 issues: auth failure and brute force
        
        // Check for brute force detection
        let brute_force = results.iter().find(|r| r.detection_type == "brute_force_attempt");
        assert!(brute_force.is_some());
        
        // Check for auth failure detection
        let auth_failure = results.iter().find(|r| r.detection_type == "auth_failure");
        assert!(auth_failure.is_some());
        
        // Check for malware indicator detection
        let malware = results.iter().find(|r| r.detection_type == "malware_indicator");
        assert!(malware.is_some());
    }

    #[test]
    fn test_file_format_detection() {
        assert_eq!(FileFormat::from_extension("csv"), FileFormat::Csv);
        assert_eq!(FileFormat::from_extension("log"), FileFormat::Log);
        assert_eq!(FileFormat::from_extension("txt"), FileFormat::Log);
        assert_eq!(FileFormat::from_extension("vmdk"), FileFormat::Vmdk);
        assert_eq!(FileFormat::from_extension("unknown"), FileFormat::Unknown);
        
        assert_eq!(FileFormat::from_path(Path::new("test.csv")), FileFormat::Csv);
        assert_eq!(FileFormat::from_path(Path::new("test.log")), FileFormat::Log);
        assert_eq!(FileFormat::from_path(Path::new("test.vmdk")), FileFormat::Vmdk);
        assert_eq!(FileFormat::from_path(Path::new("test")), FileFormat::Unknown);
    }

    #[test]
    fn test_error_handling() {
        // Test file not found error
        let analyzer = super::CsvAnalyzer::new();
        let result = analyzer.analyze(Path::new("/nonexistent/file.csv"));
        assert!(result.is_err());
        
        // Create an empty file (invalid CSV)
        let temp_file = NamedTempFile::new().unwrap();
        let result = analyzer.analyze(temp_file.path());
        assert!(result.is_ok()); // Should not error on empty file, just return empty results
        assert_eq!(result.unwrap().len(), 0);
    }
}
