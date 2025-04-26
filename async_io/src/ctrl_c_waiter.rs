#[cfg(not(test))]
use tokio::signal;

#[cfg(test)]
use mockall::{automock, mock};

#[cfg_attr(test, automock)]
pub trait CtrlCWaiter {
    async fn ctrl_c_pressed(&self);
}

#[cfg(not(test))]
#[derive(Default)]
pub struct CtrlCWaiterImpl {}

#[cfg(not(test))]
impl CtrlCWaiter for CtrlCWaiterImpl {
    async fn ctrl_c_pressed(&self) {
        signal::ctrl_c().await.unwrap();
    }
}

#[cfg(test)]
mock! {
    pub AsyncMockCtrlWaiter {}
    impl Clone for AsyncMockCtrlWaiter {
        fn clone(&self) -> Self;
    }
    impl CtrlCWaiter for AsyncMockCtrlWaiter {
        // This implementation of the mock trait method is required to allow the mock methods to return a future.
        fn ctrl_c_pressed(
            &self,
        )-> impl std::future::Future<Output=()> + Send;
    }
}
