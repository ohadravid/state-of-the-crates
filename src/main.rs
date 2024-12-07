
// #[doc = include_str!("../../post.md")]
// pub struct ReadmeDoctests;

mod connectivity;
mod logging_and_tracing;
mod error_handling;
mod testing;
mod serialization;
mod hashing;
mod allocators;
mod cli;
mod date_and_time;
mod lightning_round_ii;
mod rip;
mod orms_and_sql;
mod vec_array_ml;
mod metrics;

mod uuids {
    #[cfg(test)]
    mod tests {
        #[test]
        fn test_uuid() {
            let uuid = uuid::Uuid::from_u128(0x1337_0042);
            assert_eq!(uuid.to_string(), "00000000-0000-0000-0000-000013370042");
            assert_eq!(uuid, uuid::uuid!("00000000-0000-0000-0000-000013370042"));
        }
    }
}

fn main() {
    // connectivity::axum_example::main();
    // connectivity::warp_example::main();
    // connectivity::reqwest_example::main();
    // logging_and_tracing::main();
    // error_handling::main();
    // cli::main();
    allocators::main();
    // date_and_time::chrono_example();
    // lightning_round_ii::main();
    // rip::main();
}
