use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct JobQueue<Q, A> {
  job_in: mpsc::UnboundedReceiver<JobReq<Q, A>>,
}

impl<Q, A> JobQueue<Q, A> {
  pub fn new(job_in: mpsc::UnboundedReceiver<JobReq<Q, A>>) -> Self {
    Self { job_in }
  }

  pub fn recv(&mut self) -> Result<JobReq<Q, A>, mpsc::error::TryRecvError> {
    self.job_in.try_recv()
  }
}

#[derive(Debug)]
pub struct JobReq<Q, A> {
  req: Q,
  callback: oneshot::Sender<A>,
}

impl<Q, A> JobReq<Q, A>
where
  A: core::fmt::Debug,
{
  pub fn send(req: Q, sender: &mut mpsc::UnboundedSender<Self>) -> JobRes<A> {
    let (callback, receiver) = oneshot::channel();

    let job_req = Self { req, callback };
    let _ = sender.send(job_req);

    JobRes {
      res: None,
      receiver,
    }
  }

  pub fn reply(self, res: A) {
    self.callback.send(res).expect("failed to send reply");
  }

  pub fn req(&self) -> &Q {
    &self.req
  }
}

#[derive(Debug)]
pub struct JobRes<A> {
  res: Option<A>,
  receiver: oneshot::Receiver<A>,
}

impl<A> JobRes<A> {
  pub async fn recv(self) -> Result<A, oneshot::error::RecvError> {
    self.receiver.await
  }
}

#[cfg(test)]
mod test {
  use super::*;

  type Q = usize;
  type A = usize;

  async fn respond(queue: &mut JobQueue<Q, A>) {
    while let Ok(job_req) = queue.recv() {
      let req = job_req.req.saturating_add(1);
      job_req.reply(req);
    }
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
  async fn test_simple_ping() {
    let (mut sender, receiver) = mpsc::unbounded_channel::<JobReq<Q, A>>();
    let mut job_queue = JobQueue::new(receiver);

    let res = JobReq::send(0, &mut sender);
    respond(&mut job_queue).await;

    assert_eq!(res.recv().await, Ok(1));
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn test_simple_ping_parallel() {
    let (mut sender, receiver) = mpsc::unbounded_channel::<JobReq<Q, A>>();
    let mut job_queue = JobQueue::new(receiver);

    let res = JobReq::send(0, &mut sender);
    let res2 = JobReq::send(1, &mut sender);
    respond(&mut job_queue).await;
    respond(&mut job_queue).await;

    assert_eq!(res.recv().await, Ok(1));
    assert_eq!(res2.recv().await, Ok(2));
  }
}
