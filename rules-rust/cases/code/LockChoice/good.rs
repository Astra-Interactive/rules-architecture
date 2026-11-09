struct Mutex<T>(T);
struct Shared { flag: Mutex<bool> }

