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


enum Decision {
	Confirm,
	Quit
}

async fn b(
	mut s_to_b: UnboundedReceiver<i32>,
	b_to_s: UnboundedSender<Decision>,
	mut a_to_b: UnboundedReceiver<i32>,
	b_to_a: UnboundedSender<Decision>
)
-> Result<(), Box<dyn Error>> {
        let quote = s_to_b.next().await.ok_or(ReceiveError::EmptyStream)?;
        let participation = a_to_b.next().await.ok_or(ReceiveError::EmptyStream)?;
        if quote == participation {
            // Accept command if both prices are the same
            b_to_a.unbounded_send(Decision::Confirm)?;
            b_to_s.unbounded_send(Decision::Confirm)?;
            let _date = s_to_b.next().await.ok_or(ReceiveError::EmptyStream)?;
            println!("Accept order (price {})", quote);
            Ok(())
        } else {
            b_to_a.unbounded_send(Decision::Quit)?;
            b_to_s.unbounded_send(Decision::Quit)?;
            println!("Reject order (price inconsistency {} vs {})", quote, participation);
            Ok(())
        }
}

async fn s(
	s_to_a: UnboundedSender<i32>,
	mut a_to_s: UnboundedReceiver<i32>,
	s_to_b: UnboundedSender<i32>,
	mut b_to_s: UnboundedReceiver<Decision>,
)
-> Result<(), Box<dyn Error>> {
        let _request = a_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
        s_to_a.unbounded_send(100)?;
        s_to_b.unbounded_send(100)?;
	let decision = b_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
        match decision {
            Decision::Confirm => {
                s_to_b.unbounded_send(10)?;
                Ok(())
            }
            Decision::Quit => Ok(()),
        }
}

async fn a(
	mut s_to_a: UnboundedReceiver<i32>,
	a_to_s: UnboundedSender<i32>,
	mut b_to_a: UnboundedReceiver<Decision>,
	a_to_b: UnboundedSender<i32>)
-> Result<(), Box<dyn Error>> {
        a_to_s.unbounded_send(42)?;
	let _quote = s_to_a.next().await.ok_or(ReceiveError::EmptyStream)?;
        a_to_b.unbounded_send(42)?;
	let decision = b_to_a.next().await.ok_or(ReceiveError::EmptyStream)?;
        match decision {
		Decision::Confirm => Ok(()),
		Decision::Quit => Ok(()),
        }
}

fn main() {
	let (s_to_a_snd, s_to_a_rcv) = unbounded();
	let (a_to_s_snd, a_to_s_rcv) = unbounded();
	let (s_to_b_snd, s_to_b_rcv) = unbounded();
	let (b_to_s_snd, b_to_s_rcv) = unbounded();
	let (a_to_b_snd, a_to_b_rcv) = unbounded();
	let (b_to_a_snd, b_to_a_rcv) = unbounded();
	executor::block_on(async {
		try_join!(
		b(s_to_b_rcv, b_to_s_snd, a_to_b_rcv, b_to_a_snd),
		s(s_to_a_snd, a_to_s_rcv, s_to_b_snd, b_to_s_rcv),
		a(s_to_a_rcv, a_to_s_snd, b_to_a_rcv, a_to_b_snd)
	).unwrap();
	});
}
