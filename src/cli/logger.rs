pub enum LogKind {
    Error,
    Info,
    Warning,
}

impl std::fmt::Display for LogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "\x1b[31mERROR\x1b[0m"),
            Self::Info => write!(f, "\x1b[34mINFO\x1b[0m"),
            Self::Warning => write!(f, "\x1b[33mWARN\x1b[0m"),
        }
    }
}

static LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

pub async fn log(kind: LogKind, s: &str) {
    let _guard = LOCK.lock().await;
    let s = format!(
        "\x1b[90m[{}]\x1b[0m {}: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        kind,
        s,
    );
    if matches!(kind, LogKind::Error) {
        eprintln!("{s}");
    } else {
        println!("{s}");
    }
}

pub async fn error(s: &str) {
    log(LogKind::Error, s).await;
}

pub async fn info(s: &str) {
    log(LogKind::Info, s).await;
}

pub async fn warning(s: &str) {
    log(LogKind::Warning, s).await;
}

pub async fn log_client(prefix: &str, player: &crate::game::Player, message: &crate::messages::Message) {
    info(&format!(
        "{} \x1b[0;35m{}\x1b[0;0m (\x1b[0;36m{}\x1b[0;0m): {}",
        prefix,
        if player.username.is_empty() { "?" } else { &player.username },
        player.client.addr,
        message,
    )).await;
}

pub async fn log_from_client(player: &crate::game::Player, message: &crate::messages::Message) {
    log_client("From", player, message).await;
}

pub async fn log_to_client(player: &crate::game::Player, message: &crate::messages::Message) {
    log_client("To", player, message).await;
}
