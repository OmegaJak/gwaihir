use serde::{Deserialize, Serialize};
use serde_versioner::version;

#[derive(Serialize, Deserialize)]
#[version(crate::ThingyV1)]
struct Thingy {
    a: i32,
    b: u64,
}

fn main() {}
