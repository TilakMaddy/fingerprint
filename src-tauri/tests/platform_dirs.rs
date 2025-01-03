#[cfg(test)]
mod platform_directories_tests {

    #[test]
    fn test_platform_dirs_for_mac() {
        let app_data = platform_dirs::AppDirs::new(Some("TheTunnel"), true);
        assert!(app_data.is_some());
    }
}
