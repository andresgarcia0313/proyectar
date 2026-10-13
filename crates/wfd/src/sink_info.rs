//! What the sink told about itself in M3 beyond formats, for the log and the interface.

use crate::caps::MANDATORY;
use crate::params;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SinkInfo {
    /// `intel_friendly_name`: the name the TV shows for itself.
    pub friendly_name: Option<String>,
    /// `wfd_connector_type`, e.g. `05` for HDMI, `07` for an embedded panel.
    pub connector_type: Option<String>,
    pub preferred_display_mode: Option<String>,
    /// `wfd_display_edid`: block count and hex EDID, as the sink sent it.
    pub edid: Option<String>,
    /// `wfd_standby_resume_capability: supported`: the source may send `wfd_standby`.
    pub standby_resume: bool,
    /// Every other parameter (`microsoft_*`, `intel_*`, `wfd2_*`...), untouched.
    pub other: Vec<(String, String)>,
}

impl SinkInfo {
    pub fn from_caps(caps: &str) -> Self {
        let field = |name: &str| {
            params::get(caps, name)
                .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none"))
                .map(str::to_owned)
        };
        let known = [
            "intel_friendly_name",
            "wfd_connector_type",
            "wfd_preferred_display_mode",
            "wfd_display_edid",
            "wfd_standby_resume_capability",
        ];
        let other = caps
            .lines()
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim(), v.trim()))
            .filter(|(k, _)| {
                !k.is_empty() && !MANDATORY.iter().chain(&known).any(|n| n.eq_ignore_ascii_case(k))
            })
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Self {
            friendly_name: field("intel_friendly_name"),
            connector_type: field("wfd_connector_type"),
            preferred_display_mode: field("wfd_preferred_display_mode"),
            edid: field("wfd_display_edid"),
            standby_resume: field("wfd_standby_resume_capability")
                .is_some_and(|v| v.eq_ignore_ascii_case("supported")),
            other,
        }
    }
}
