fn work(values: Vec<u64>) {
let result = values.into_iter().map(first).map(second).map(third).collect::<Vec<_>>();
} fn first(value:u64)->u64 {value} fn second(value:u64)->u64 {value} fn third(value:u64)->u64 {value}
