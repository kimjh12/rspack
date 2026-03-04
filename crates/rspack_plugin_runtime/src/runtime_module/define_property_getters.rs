use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static DEFINE_PROPERTY_GETTERS_TEMPLATE: &str = include_str!("runtime/define_property_getters.ejs");
static DEFINE_PROPERTY_GETTERS_PER_EXPORT_TEMPLATE: &str =
  include_str!("runtime/define_property_getters_per_export.ejs");
static DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(DEFINE_PROPERTY_GETTERS_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefinePropertyGettersRuntimeModule {
  batch: bool,
}

impl DefinePropertyGettersRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, batch: bool) -> Self {
    Self::with_default(runtime_template, batch)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for DefinePropertyGettersRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    let template = if self.batch {
      DEFINE_PROPERTY_GETTERS_TEMPLATE
    } else {
      DEFINE_PROPERTY_GETTERS_PER_EXPORT_TEMPLATE
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
    *DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS
  }
}
