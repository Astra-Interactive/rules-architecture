fn run(state: tokio::sync::watch::Sender<u32>) {
    let mut captured = None;
    state.send_modify(|current| { captured = Some(*current); *current += 1; });
}

