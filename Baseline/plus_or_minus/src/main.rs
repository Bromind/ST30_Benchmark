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

enum GuessResult {
	More,
	Less,
	Correct
}


async fn a(a_to_b: UnboundedSender<i32>) -> Result<(), Box<dyn Error>> {
	a_to_b.unbounded_send(10)?;
	return Ok(())
}

async fn b(mut a_to_b: UnboundedReceiver<i32>,
	mut c_to_b: UnboundedReceiver<i32>,
	b_to_c: UnboundedSender<GuessResult>)
	-> Result<(), Box<dyn Error>>
{
        let _n = a_to_b.next().await.ok_or(ReceiveError::EmptyStream)?;
        let _x = c_to_b.next().await.ok_or(ReceiveError::EmptyStream)?;
// Assuming it is correct
	b_to_c.unbounded_send(GuessResult::Correct)?;
        return Ok(())
}

async fn c(mut b_to_c: UnboundedReceiver<GuessResult>,
	c_to_b: UnboundedSender<i32>)
	-> Result<(), Box<dyn Error>> {
        c_to_b.unbounded_send(10)?;
	let r = b_to_c.next().await.ok_or(ReceiveError::EmptyStream)?;
        match r {
            GuessResult::Correct => return Ok(()),
            _ => panic!(),
        }
}

fn main() {
	let (a_to_b_snd, a_to_b_rcv) = unbounded();
	let (c_to_b_snd, c_to_b_rcv) = unbounded();
	let (b_to_c_snd, b_to_c_rcv) = unbounded();
	executor::block_on(async {
	try_join!(
		a(a_to_b_snd),
		b(a_to_b_rcv, c_to_b_rcv, b_to_c_snd),
		c(b_to_c_rcv, c_to_b_snd)
	).unwrap();
	});
}
