#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Env {
    Production,
    Development,
    Test,
}

impl Env {
    pub fn is_test(self) -> bool {
        matches!(self, Env::Test)
    }
}

pub fn current() -> Env {
    match std::env::var("APP_ENV").unwrap().as_str() {
        "production" => Env::Production,
        "development" => Env::Development,
        "test" => Env::Test,
        _ => panic!("Unknown Evironment"),
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn in_test_env() {
        std::env::set_var("APP_ENV", "test");
        let env = current();

        assert!(env.is_test());
    }
}