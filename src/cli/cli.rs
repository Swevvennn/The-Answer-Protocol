pub enum Action<T> {
    Awake,
    Interrupt,
    Other(T),
    Validate,
}

pub trait Wrapper {
    type OtherAction;

    fn new() -> Self;
    fn draw(&self, terminal: &mut crate::cli::Terminal) -> impl std::future::Future<Output = ()> + Send;
    fn select(&mut self, terminal: &mut crate::cli::Terminal) -> impl std::future::Future<Output = Option<Action<Self::OtherAction>>> + Send;
    fn process(&mut self, action: Action<Self::OtherAction>) -> impl std::future::Future<Output = Result<(), std::io::Error>> + Send;
    fn update(&mut self) -> impl std::future::Future<Output = ()> + Send;
}

pub async fn run<T: Wrapper>() {
    let mut terminal = match crate::cli::Terminal::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: failed to create terminal ui: {e}");
            return;
        }
    };
    let mut wrapper = T::new();
    loop {
        wrapper.draw(&mut terminal).await;
        if let Err(e) = match wrapper.select(&mut terminal).await {
            Some(action) => match action {
                Action::Interrupt => Err(std::io::Error::other("Interrupted")),
                action => wrapper.process(action).await,
            }
            None => Ok(()),
        } {
            if let Err(e) = terminal.close() {
                eprintln!("Error: failed to close terminal ui: {e}");
            };
            eprintln!("{e}");
            return;
        }
        wrapper.update().await;
    }
}
