use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, RuntimeTemplate, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CompatGetDefaultExportRuntimeModule {
  id: Identifier,
  batch: bool,
}

impl CompatGetDefaultExportRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, batch: bool) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}compat_get_default_export",
        runtime_template.runtime_module_prefix()
      )),
      batch,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CompatGetDefaultExportRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      if self.batch {
        include_str!("runtime/compat_get_default_export.ejs").to_string()
      } else {
        include_str!("runtime/compat_get_default_export_per_export.ejs").to_string()
      },
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }
}
