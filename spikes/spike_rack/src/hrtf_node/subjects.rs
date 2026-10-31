//! Vendored from firewheel-ircam-hrtf 0.5.0 (MIT OR Apache-2.0), trimmed to the
//! default IRCAM subject for the spike.

use std::sync::Arc;

use firewheel::collector::ArcGc;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    #[default]
    Irc1040,
}

impl AsRef<[u8]> for Subject {
    fn as_ref(&self) -> &[u8] {
        match self {
            Subject::Irc1040 => include_bytes!("../../assets/irc_1040_c.bin"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubjectBytes(ArcGc<[u8]>);

impl SubjectBytes {
    pub fn new(bytes: &[u8]) -> Self {
        Self(ArcGc::new_unsized(|| Arc::from(bytes)))
    }
}

impl AsRef<[u8]> for SubjectBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
