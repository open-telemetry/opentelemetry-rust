// Each transform is written once and mounted for every generated type tree.
#![allow(clippy::duplicate_mod)]

#[cfg(feature = "gen-tonic-messages")]
#[path = ""]
mod tonic_tree {
    use crate::proto::tonic as proto;
    pub mod common;
    #[cfg(feature = "logs")]
    pub mod logs;
    #[cfg(feature = "metrics")]
    pub mod metrics;
    #[cfg(feature = "trace")]
    pub mod trace;
}

#[cfg(feature = "gen-json")]
#[path = ""]
mod json_tree {
    use crate::proto::json as proto;
    pub mod common;
    #[cfg(feature = "logs")]
    pub mod logs;
    #[cfg(feature = "metrics")]
    pub mod metrics;
    #[cfg(feature = "trace")]
    pub mod trace;
}

pub mod common {
    #[cfg(feature = "gen-json")]
    pub use super::json_tree::common as json;
    #[cfg(feature = "gen-tonic-messages")]
    pub use super::tonic_tree::common as tonic;
}

#[cfg(feature = "metrics")]
pub mod metrics {
    #[cfg(feature = "gen-json")]
    pub use super::json_tree::metrics as json;
    #[cfg(feature = "gen-tonic-messages")]
    pub use super::tonic_tree::metrics as tonic;
}

#[cfg(feature = "trace")]
pub mod trace {
    #[cfg(feature = "gen-json")]
    pub use super::json_tree::trace as json;
    #[cfg(feature = "gen-tonic-messages")]
    pub use super::tonic_tree::trace as tonic;
}

#[cfg(feature = "logs")]
pub mod logs {
    #[cfg(feature = "gen-json")]
    pub use super::json_tree::logs as json;
    #[cfg(feature = "gen-tonic-messages")]
    pub use super::tonic_tree::logs as tonic;
}

#[cfg(feature = "zpages")]
pub mod tracez;

#[cfg(feature = "profiles")]
pub mod profiles;
