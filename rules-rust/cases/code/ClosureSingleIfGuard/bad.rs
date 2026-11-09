fn run() {
    spawn(async {
        if ready { first(); second(); }
    });
    consume(|| { if ready { first(); second(); } });
    consume(|| if ready { first(); second(); });
}
