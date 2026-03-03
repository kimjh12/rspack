use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, RuntimeTemplate, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
  batch: bool,
}

impl CreateFakeNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, batch: bool) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}create_fake_namespace_object",
        runtime_template.runtime_module_prefix()
      )),
      batch,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      if self.batch {
        include_str!("runtime/create_fake_namespace_object.ejs").to_string()
      } else {
        include_str!("runtime/create_fake_namespace_object_per_export.ejs").to_string()
      },
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }
}
