pub enum VisualizationError {
  Warning(String),
}

impl<'a> From<&'a str> for VisualizationError {
  fn from(err: &'a str) -> VisualizationError {
    VisualizationError::Warning(String::from(err))
  }
}
