use std::collections::HashMap;
use std::sync::LazyLock;

static HASHMAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "foo");
    m.insert(1, "bar");
    m.insert(2, "baz");
    m
});

use std::sync::OnceLock;
static CELL: OnceLock<usize> = OnceLock::new();

pub fn main() {
    dbg!(&*HASHMAP);

    let value = CELL.get_or_init(|| 12345);
}
