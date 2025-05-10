pub use turbo_program::*;
pub mod proof;
pub mod proof_worker;
pub mod prove_queue;
pub mod server;
pub mod session;
pub mod session_manager;
pub mod session_simple;
pub mod warp;
pub mod zeromind;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
