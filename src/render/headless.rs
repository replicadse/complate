use {
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
impl super::UserInput for HeadlessBackend {
    async fn prompt(&self, _text: &str) -> Result<String> {
        Err(anyhow::anyhow!("can not prompt in headless backend").into())
    }

    async fn select(
        &self,
        _prompt: &str,
        _options: &std::collections::BTreeMap<String, crate::config::Option>,
    ) -> Result<String> {
        Err(anyhow::anyhow!("can not prompt in headless backend").into())
    }

    async fn check(
        &self,
        _prompt: &str,
        _separator: &str,
        _options: &std::collections::BTreeMap<String, crate::config::Option>,
    ) -> Result<String> {
        Err(anyhow::anyhow!("can not prompt in headless backend").into())
    }
}
