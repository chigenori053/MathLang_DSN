//! ProblemIntent Foundation layer. The MIRP state schema is unchanged.

pub mod parser;
pub mod runtime;
pub mod schema;

pub use parser::{parse_problem, ParseOutcome};
pub use runtime::*;
pub use schema::*;
