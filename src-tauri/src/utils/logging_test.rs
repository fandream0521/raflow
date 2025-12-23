#[cfg(test)]
mod tests {
    use super::super::logging::init_logging;

    #[test]
    fn test_logging_initialization() {
        // This test ensures the logging system can be initialized without panicking
        // Note: We can only call init_logging once per process, so we need to handle
        // the case where it's already been called

        // Just verify the function exists and can be called
        // In a real test, we would capture logs, but that's beyond the scope of this basic test
        init_logging();

        // If we get here without panicking, the test passes
        assert!(true);
    }
}
