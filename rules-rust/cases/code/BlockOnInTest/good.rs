fn run() { futures::executor::block_on(work()); }
#[cfg(test)]
mod test { fn work() {}
    #[tokio::test]
    async fn run() { work().await; }
}
fn work() {}
