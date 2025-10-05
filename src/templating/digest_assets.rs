use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderErrorReason,
};

pub(crate) struct DigestAssetHandlebarsHelper {
    pub(crate) cache_key: u64,
}

impl HelperDef for DigestAssetHandlebarsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let file = h
            .param(0)
            .map(|v| v.value())
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("digest_asset", 0))?;

        let mut path = "/assets/".to_string();

        path.push_str(&file.to_string().replace("\"", ""));
        path.push_str("?v=");
        path.push_str(&self.cache_key.to_string());

        out.write(&path)?;
        Ok(())
    }
}
