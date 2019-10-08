#[derive(Debug)]
/// Definition of the possible errors that can appear during
/// the execution of all of the crate functions related to the
/// permutation process.
pub enum PermError {
    /// This is returned if you try to add more input values into the width than
    /// it can hold
    InputFull,
    /// This is returned if an odd number of full rounds is specified. This implementation only supports
    /// the symmetric Permutation variant.
    FullRoundsOdd,
    /// This error occurs when you try to invert a scalar which has the value of zero
    NonInvertible,
    /// This errors occurs when a user tries to fetch a constant and the iterator function returns `None`
    NoMoreConstants,
}
