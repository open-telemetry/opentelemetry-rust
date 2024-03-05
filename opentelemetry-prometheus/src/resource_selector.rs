use crate::get_attrs;
use opentelemetry::Key;
use opentelemetry_sdk::Resource;
use prometheus::proto::LabelPair;
use std::collections::HashSet;

/// `ResourceSelector` is used to select which resource to export with every metrics.
///
/// By default, the exporter will only export resource as `target_info` metrics but not inline in every
/// metrics. You can disable this behavior by calling [`ExporterBuilder::without_target_info`].
///
/// You can add resource to every metrics by set `ResourceSelector` to anything other than `None`.
///
/// By default, ResouceSelector is `None`.
#[derive(Debug, Default)]
#[non_exhaustive]
pub enum ResourceSelector {
    /// Export all resource attributes with every metrics.
    All,
    /// Do not export any resource attributes with every metrics.
    #[default]
    None,
    /// Export only the resource attributes in the allow list with every metrics.
    KeyAllowList(HashSet<Key>),
}

impl From<HashSet<Key>> for ResourceSelector {
    fn from(keys: HashSet<Key>) -> Self {
        ResourceSelector::KeyAllowList(keys)
    }
}

impl ResourceSelector {
    pub(crate) fn select(&self, resource: &Resource) -> Vec<LabelPair> {
        match self {
            ResourceSelector::All => get_attrs(&mut resource.iter(), &[]),
            ResourceSelector::None => Vec::new(),
            ResourceSelector::KeyAllowList(keys) => {
                get_attrs(&mut resource.iter().filter(|(k, _)| keys.contains(k)), &[])
            }
        }
    }
}
