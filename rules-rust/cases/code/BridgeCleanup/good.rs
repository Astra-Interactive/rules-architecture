struct Guard;
impl Guard { fn drop(&mut self) { let handle = tokio::spawn(async {}); } }

