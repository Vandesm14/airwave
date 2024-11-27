use engine::command::CommandWithFreq;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct JobQueue {
  job_in: mpsc::UnboundedReceiver<JobReq>,
}

impl JobQueue {
  pub fn new(job_in: mpsc::UnboundedReceiver<JobReq>) -> Self {
    Self { job_in }
  }

  pub fn recv(&mut self) -> Result<JobReq, mpsc::error::TryRecvError> {
    self.job_in.try_recv()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobReqKind {
  Ping,

  // GET
  Messages,

  // POST
  Command(CommandWithFreq),
}

#[derive(Debug)]
pub struct JobReq {
  req: JobReqKind,
  callback: oneshot::Sender<JobResKind>,
}

impl JobReq {
  pub fn send(
    req: JobReqKind,
    sender: &mut mpsc::UnboundedSender<Self>,
  ) -> JobRes {
    let (callback, receiver) = oneshot::channel();

    let job_req = Self { req, callback };
    let _ = sender.send(job_req);

    JobRes {
      res: None,
      receiver,
    }
  }

  pub fn reply(self, res: JobResKind) {
    self.callback.send(res).unwrap();
  }

  pub fn req(&self) -> &JobReqKind {
    &self.req
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobResKind {
  Pong,

  // GET
  Messages(Vec<CommandWithFreq>),
}

#[derive(Debug)]
pub struct JobRes {
  res: Option<JobResKind>,
  receiver: oneshot::Receiver<JobResKind>,
}

impl JobRes {
  pub async fn recv(self) -> Result<JobResKind, oneshot::error::RecvError> {
    self.receiver.await
  }
}

#[cfg(test)]
mod test {
  use super::*;

  async fn respond(queue: &mut JobQueue) {
    while let Ok(job_req) = queue.recv() {
      match job_req.req {
        JobReqKind::Ping => job_req.reply(JobResKind::Pong),
        _ => {}
      }
    }
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
  async fn test_simple_ping() {
    let (mut sender, receiver) = mpsc::unbounded_channel::<JobReq>();
    let mut job_queue = JobQueue::new(receiver);

    let res = JobReq::send(JobReqKind::Ping, &mut sender);
    respond(&mut job_queue).await;

    assert_eq!(res.recv().await, Ok(JobResKind::Pong));
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn test_simple_ping_parallel() {
    let (mut sender, receiver) = mpsc::unbounded_channel::<JobReq>();
    let mut job_queue = JobQueue::new(receiver);

    let res = JobReq::send(JobReqKind::Ping, &mut sender);
    let res2 = JobReq::send(JobReqKind::Ping, &mut sender);
    respond(&mut job_queue).await;
    respond(&mut job_queue).await;

    assert_eq!(res.recv().await, Ok(JobResKind::Pong));
    assert_eq!(res2.recv().await, Ok(JobResKind::Pong));
  }
}
