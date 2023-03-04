use crate::error::Error;

use super::UserInput;
use async_trait::async_trait;
use std::result::Result;

pub struct HeadlessBackend {}

impl HeadlessBackend {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UserInput for HeadlessBackend {
    async fn prompt(&self, _text: &str) -> Result<String, Box<dyn std::error::Error>> {
        Err(Box::new(Error::Generic("can not prompt in headless backend".into())))
    }

    async fn select(
        &self,
        _prompt: &str,
        _options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err(Box::new(Error::Generic("can not prompt in headless backend".into())))
    }

    async fn check(
        &self,
        _prompt: &str,
        _separator: &str,
        _options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err(Box::new(Error::Generic("can not prompt in headless backend".into())))
    }
}
