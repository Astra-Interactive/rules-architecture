fn run(result: Result<(), std::io::Error>) {
    if let Err(error) = result { tracing::warn!(error = &error as &dyn std::error::Error, "Failed"); }
}

