use std::convert::TryFrom;

/// An error that is returned from status associated methods when the received status is unknown.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidStatus(String);

/// Combined status for a specific ref.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CombinedStatus {
    Success,
    Pending,
    Failure,
}

impl<'a> TryFrom<&'a str> for CombinedStatus {
    type Error = InvalidStatus;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "success" => Ok(CombinedStatus::Success),
            "pending" => Ok(CombinedStatus::Pending),
            "failure" | "error" => Ok(CombinedStatus::Failure),
            unknown => Err(InvalidStatus(unknown.into())),
        }
    }
}
