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
	Success,
	Failure
}

async fn C(
	c_to_s: UnboundedSender<i32>,
	mut s_to_c: UnboundedReceiver<Decision>,
)
-> Result<(), Box<dyn Error>> {
	c_to_s.unbounded_send(10000000)?;
	let mut cur_attempt = 0;
	loop {
		c_to_s.unbounded_send(cur_attempt)?;
		cur_attempt += 1;
		let pwd = s_to_c.next().await.ok_or(ReceiveError::EmptyStream)?;
		match pwd {
			Decision::Success => {
				println!("Success");
				return Ok(());
			}
			Decision::Failure => {
				println!("Failure");
			}
		}
	}
}

async fn S(
	mut c_to_s: UnboundedReceiver<i32>,
	s_to_c: UnboundedSender<Decision>,
) -> Result<(), Box<dyn Error>> {
        let password = c_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
        loop {
            let attempt = c_to_s.next().await.ok_or(ReceiveError::EmptyStream)?;
            if attempt == password {
                s_to_c.unbounded_send(Decision::Success)?;
                return Ok(());
            } else {
                s_to_c.unbounded_send(Decision::Failure)?;
            }
        }
}

fn main() {
	let (s_to_c_snd, s_to_c_rcv) = unbounded();
	let (c_to_s_snd, c_to_s_rcv) = unbounded();
	executor::block_on(async {
			try_join!(C(c_to_s_snd, s_to_c_rcv), S(c_to_s_rcv, s_to_c_snd)).unwrap();
	});
}
