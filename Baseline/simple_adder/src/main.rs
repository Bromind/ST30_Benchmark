use futures::channel::mpsc::*;
use futures::stream::StreamExt;
use futures::executor;
use futures::try_join;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReceiveError {
    #[error("receiver stream is empty")]
    EmptyStream,
}

async fn c(c_to_s: UnboundedSender<i32>, mut s_to_c: UnboundedReceiver<i32>) -> Result<(), Box<dyn Error>> {
	c_to_s.unbounded_send(10)?;
	c_to_s.unbounded_send(10)?;
	let result = s_to_c.next().await.ok_or(ReceiveError::EmptyStream)?;

	println!("{:?}", result);
	return Ok(())
}

async fn s(s_to_c: UnboundedSender<i32>, mut c_to_s: UnboundedReceiver<i32>) -> Result<(), Box<dyn Error>> {
        let x = c_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
        let y = c_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
        s_to_c.unbounded_send(x + y)?;
        return Ok(())
}

fn main() {
	let (s_to_c_snd, s_to_c_rcv) = unbounded();
	let (c_to_s_snd, c_to_s_rcv) = unbounded();
	executor::block_on(async {
		try_join!(s(s_to_c_snd, c_to_s_rcv), c(c_to_s_snd, s_to_c_rcv)).unwrap();
	});
}
