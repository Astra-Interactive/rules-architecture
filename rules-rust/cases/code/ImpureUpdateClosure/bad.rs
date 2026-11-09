fn run(counter: &std::sync::atomic::AtomicUsize) {
    let mut captured = 0;
    counter.fetch_update(std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed, |current| { captured = current; Some(current + 1) });
}

