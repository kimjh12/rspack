use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/create_fake_namespace_object.ejs");
static CREATE_FAKE_NAMESPACE_OBJECT_PER_EXPORT_TEMPLATE: &str =
  include_str!("runtime/create_fake_namespace_object_per_export.ejs");
static CREATE_FAKE_NAMESPACE_OBJECT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  batch: bool,
}

impl CreateFakeNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, batch: bool) -> Self {
    Self::with_default(runtime_template, batch)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    let template = if self.batch {
      CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE
    } else {
      CREATE_FAKE_NAMESPACE_OBJECT_PER_EXPORT_TEMPLATE
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
    *CREATE_FAKE_NAMESPACE_OBJECT_RUNTIME_REQUIREMENTS
  }
}
