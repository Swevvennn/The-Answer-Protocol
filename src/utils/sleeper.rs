pub struct Awaker(crate::utils::Shared<tokio::sync::watch::Sender<()>>);

impl Awaker {
    pub fn new(sender: tokio::sync::watch::Sender<()>) -> Self {
        Self(
            crate::utils::Shared::new(
                sender
            )
        )
    }

    pub fn clone(&self) -> Awaker {
        Awaker(self.0.clone())
    }

    pub async fn wake(&self) {
        let _ = self.0.lock().await.send(());
    }
}

pub struct Sleeper {
    receiver: tokio::sync::watch::Receiver<()>,
    awaker: Awaker,
}

impl Sleeper {
    pub fn new_awaker(&self) -> Awaker {
        self.awaker.clone()
    }

    pub async fn wait(&mut self) {
        let _ = self.receiver.changed().await;
    }
}

impl Default for Sleeper {
    fn default() -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(());
        Self {
            receiver,
            awaker: Awaker::new(sender),
        }
    }
}
