/// Total ammount of full rounds that will be applied.
/// This is expressed as `RF` in the paper.
pub(crate) const TOTAL_FULL_ROUNDS: usize = 8;

/// Total ammount of partial rounds that will be applied.
/// This is expressed as `Rp` in the paper.
pub(crate) const PARTIAL_ROUNDS: usize = 59;

include!(concat!(env!("OUT_DIR"), "/width.rs"));
