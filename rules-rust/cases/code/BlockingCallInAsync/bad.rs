use std::thread::sleep as pause;
async fn run() {
    pause(std::time::Duration::ZERO);
    fn sync() { pause(std::time::Duration::ZERO); }
    let sync = || pause(std::time::Duration::ZERO);
    let future = async { std::fs::read("file"); };
}

