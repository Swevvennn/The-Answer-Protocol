#[derive(Default)]
pub struct Input {
    pub input: String,
}

impl Input {
    pub fn consume(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        input
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
