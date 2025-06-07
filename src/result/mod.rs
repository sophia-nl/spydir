mod diff;
mod last;
mod new;

pub use diff::Diff;
pub use diff::DiffResult::{Changed, NoChange};
pub use last::GetLastResult;
pub use new::{CreateNewResult, NewResult, WriteToFile};
