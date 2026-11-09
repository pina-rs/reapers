//! A scene's dependent host calls run only after the previous reply.

use std::cell::RefCell;
use std::future::Future;

use futures::FutureExt;
use futures::future::LocalBoxFuture;
use futures::future::Shared;
use leptoskit::scene3d::client::ClientError;

type Completion = Shared<LocalBoxFuture<'static, Result<(), ClientError>>>;

#[derive(Default)]
pub struct Submissions {
	tail: RefCell<Option<Completion>>,
}

impl Submissions {
	pub fn after<F, Task>(&self, send: F) -> Completion
	where
		F: FnOnce() -> Task + 'static,
		Task: Future<Output = Result<(), ClientError>> + 'static,
	{
		let previous = self.tail.borrow().clone();
		let next = async move {
			if let Some(previous) = previous {
				// Each caller observes its own result. In particular, a
				// failed batch must still allow the scene to be closed.
				let _ = previous.await;
			}
			send().await
		}
		.boxed_local()
		.shared();
		*self.tail.borrow_mut() = Some(next.clone());
		next
	}

	pub async fn ready(&self) -> Result<(), ClientError> {
		let tail = self.tail.borrow().clone();
		match tail {
			Some(tail) => tail.await,
			None => Ok(()),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::cell::Cell;
	use std::rc::Rc;

	use futures::channel::oneshot;
	use futures::executor::LocalPool;
	use futures::task::LocalSpawnExt;

	use super::*;

	#[test]
	fn a_failed_batch_still_allows_the_scene_to_close() {
		let queue = Submissions::default();
		let _failed = queue.after(|| async { Err(ClientError::Rejected) });
		let closed = Rc::new(Cell::new(false));
		let result = closed.clone();
		let close = queue.after(move || {
			async move {
				result.set(true);
				Ok(())
			}
		});
		assert_eq!(futures::executor::block_on(close), Ok(()));
		assert!(closed.get());
	}

	#[test]
	fn loading_and_listening_wait_for_close_and_apply_acknowledgements() {
		let queue = Rc::new(Submissions::default());
		let mut pool = LocalPool::new();
		let (closed, close_reply) = oneshot::channel();
		let (applied, apply_reply) = oneshot::channel();
		let order = Rc::new(RefCell::new(Vec::new()));
		let calls = order.clone();
		let _close = queue.after(move || {
			async move {
				calls.borrow_mut().push("close");
				close_reply.await.unwrap();
				Ok(())
			}
		});
		let calls = order.clone();
		let _apply = queue.after(move || {
			async move {
				calls.borrow_mut().push("apply");
				apply_reply.await.unwrap();
				Ok(())
			}
		});
		let listening = Rc::new(Cell::new(false));
		let ready = listening.clone();
		pool.spawner()
			.spawn_local(async move {
				queue.ready().await.unwrap();
				ready.set(true);
			})
			.unwrap();
		pool.run_until_stalled();
		assert_eq!(*order.borrow(), ["close"]);
		assert!(!listening.get());
		closed.send(()).unwrap();
		pool.run_until_stalled();
		assert_eq!(*order.borrow(), ["close", "apply"]);
		assert!(!listening.get());
		applied.send(()).unwrap();
		pool.run_until_stalled();
		assert!(listening.get());
	}
}
