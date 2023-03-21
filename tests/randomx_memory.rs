extern crate lazy_static;
extern crate mithril;
use test_case::test_case;

use lazy_static::lazy_static;
use mithril::randomx::memory::{self, SeedMemory};

lazy_static! {
    static ref TEST_SEED_MEM: SeedMemory = SeedMemory::new_initialised(b"test key 000");
}

#[test_case(0, 0 => 0x191e0e1d23c02186)]
#[test_case(12253, 29 => 0xf1b62fe6210bf8b1)]
#[test_case(262143, 127 => 0x1f47f056d05cd99b)]
fn test_seed_memory_new_initialised(i: usize, j: usize) -> u64 {
    TEST_SEED_MEM.blocks[i][j]
}

#[test_case(0 => 0x680588a85ae222db)]
#[test_case(10000000 => 0x7943a1f6186ffb72)]
#[test_case(20000000 => 0x9035244d718095e1)]
#[test_case(30000000 => 0x145a5091f7853099)]
fn init_dataset_item(item_num: u64) -> u64 {
    memory::init_dataset_item(&TEST_SEED_MEM, item_num)[0]
}
