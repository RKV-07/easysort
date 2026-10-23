pub mod error;
pub mod mode;
pub mod mover;
pub mod plan;
pub mod rules;
pub mod undo;

pub use error::Error;
pub use mode::Mode;
pub use mover::{execute, unique_name, Failure, MoveReport, MovedFile};
pub use plan::{build_plan, scan, FileEntry, Plan, PlannedMove};
pub use rules::{classify, default_rules, Rules};
pub use undo::{append_manifest, undo_batch, Batch, MovedPair, UndoStack};
