mod digest_assets;

use crate::templating::digest_assets::DigestAssetHandlebarsHelper;
use handlebars::{self, DirectorySourceOptions};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Mutex, time::SystemTime};
use thiserror::Error;

pub use handlebars::to_json;

pub struct Renderer {
    context: Mutex<BTreeMap<String, Value>>,
    #[allow(dead_code)]
    handlebars: handlebars::Handlebars<'static>,
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("template error")]
    TemplateError(#[from] handlebars::TemplateError),

    #[error("render error")]
    RenderError(#[from] handlebars::RenderError),

    #[error("context update error")]
    ContextUpdateError,
}

impl Renderer {
    pub fn new(directory: String) -> Result<Self, RendererError> {
        let cache_key = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let cache_key = cache_key.as_secs();

        let mut handlebars = handlebars::Handlebars::new();

        handlebars.set_strict_mode(true);
        handlebars.set_dev_mode(true);
        handlebars
            .register_templates_directory(directory, DirectorySourceOptions::default())
            .map_err(RendererError::TemplateError)?;

        handlebars.register_helper(
            "digest_asset",
            Box::new(DigestAssetHandlebarsHelper { cache_key }),
        );

        #[allow(unused_mut)]
        let mut context: BTreeMap<String, Value> = BTreeMap::new();

        Ok(Self {
            context: Mutex::new(context),
            handlebars,
        })
    }

    pub fn insert(&self, key: &str, value: impl Into<Value>) -> Result<(), RendererError> {
        match self.context.try_lock() {
            Ok(mut context) => {
                context.insert(key.to_string(), value.into());
                Ok(())
            }
            Err(_) => Err(RendererError::ContextUpdateError),
        }
    }

    pub fn render(&self, template_name: &str) -> Result<String, RendererError> {
        self.handlebars
            .render(template_name, &self.context)
            .map_err(RendererError::RenderError)
    }
}
