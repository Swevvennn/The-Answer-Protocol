pub struct Waiter {
    instant: Option<tokio::time::Instant>,
}

impl Waiter {
    pub fn new() -> Self {
        Self {
            instant: None,
        }
    }

    pub async fn block() {
        std::future::pending::<()>().await
    }

    pub fn is_waiting(&self) -> bool {
        !matches!(self.instant, None)
    }

    pub fn begin(&mut self) {
        self.instant = Some(tokio::time::Instant::now());
    }

    pub fn end(&mut self) {
        self.instant = None;
    }

    pub async fn wait(&mut self) {
        match self.instant {
            Some(start) => tokio::time::sleep_until(start + tokio::time::Duration::from_secs(3)).await,
            None => Self::block().await,
        };
        self.instant = None;
    }
}
