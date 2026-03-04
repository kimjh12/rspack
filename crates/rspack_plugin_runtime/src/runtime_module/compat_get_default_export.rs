use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static COMPAT_GET_DEFAULT_EXPORT_TEMPLATE: &str =
  include_str!("runtime/compat_get_default_export.ejs");
static COMPAT_GET_DEFAULT_EXPORT_PER_EXPORT_TEMPLATE: &str =
  include_str!("runtime/compat_get_default_export_per_export.ejs");
static COMPAT_GET_DEFAULT_EXPORT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(COMPAT_GET_DEFAULT_EXPORT_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct CompatGetDefaultExportRuntimeModule {
  batch: bool,
}

impl CompatGetDefaultExportRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, batch: bool) -> Self {
    Self::with_default(runtime_template, batch)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CompatGetDefaultExportRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    let template = if self.batch {
      COMPAT_GET_DEFAULT_EXPORT_TEMPLATE
    } else {
      COMPAT_GET_DEFAULT_EXPORT_PER_EXPORT_TEMPLATE
    };
    vec![(self.id.to_string(), template.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(&self.id, None)?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *COMPAT_GET_DEFAULT_EXPORT_RUNTIME_REQUIREMENTS
  }
}
