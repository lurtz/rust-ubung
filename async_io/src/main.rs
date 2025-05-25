mod async_adder;
mod ctrl_c_waiter;
mod state;
mod stdio;

#[cfg(not(test))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    async_adder::main2(
        listener,
        &ctrl_c_waiter::CtrlCWaiterImpl::default(),
        Box::new(stdio::StdioImpl::default()),
    )
    .await
}
