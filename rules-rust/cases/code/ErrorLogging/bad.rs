fn run(result: Result<(), std::io::Error>) {
    if let Err(error) = result { tracing::warn!("Failed: {error}"); }
}

