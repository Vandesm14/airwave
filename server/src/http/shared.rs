use tokio::sync::mpsc;

use crate::{
  job::JobReq,
  runner::{ArgReqKind, ResKind, TinyReqKind},
};

pub type GetSender = mpsc::UnboundedSender<JobReq<TinyReqKind, ResKind>>;
pub type PostSender = mpsc::UnboundedSender<JobReq<ArgReqKind, ResKind>>;

#[derive(Debug, Clone)]
pub struct AppState {
  pub tiny_sender: GetSender,
  pub big_sender: PostSender,
}

impl AppState {
  pub fn new(get_sender: GetSender, post_sender: PostSender) -> Self {
    Self {
      tiny_sender: get_sender,
      big_sender: post_sender,
    }
  }
}
