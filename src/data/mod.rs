// bundled NZAVS variable names for fuzzy completion

pub mod measure_workspace;
pub mod measures;
pub mod variable_metadata;
pub mod variables;

pub use measure_workspace::MeasureWorkspace;
pub use measures::MeasureFileFormat;
pub use variable_metadata::{lookup_variable_description, variable_metadata_source};
pub use variables::VARIABLES;
