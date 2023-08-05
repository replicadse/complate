use {
    super::UserInput,
    crate::error::Error,
    anyhow::Result,
    async_trait::async_trait,
};

pub struct HeadlessBackend {}

impl HeadlessBackend {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UserInput for HeadlessBackend {
    async fn prompt(&self, _text: &str) -> Result<String> {
        Err(Error::Invalid("can not prompt in headless backend".into()).into())
    }

    async fn select(
        &self,
        _prompt: &str,
        _options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String> {
        Err(Error::Invalid("can not prompt in headless backend".into()).into())
    }

    async fn check(
        &self,
        _prompt: &str,
        _separator: &str,
        _options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String> {
        Err(Error::Invalid("can not prompt in headless backend".into()).into())
    }
}
