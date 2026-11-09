fn run() {
    spawn(async { if ready { first(); } });
    consume(|| { if ready { first(); second(); } else { other(); } });
    consume(|| { if !ready { return; } first(); second(); });
}
