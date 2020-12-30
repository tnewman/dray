pub struct Status {
    pub id: u32,
    pub status_code: StatusCode,
}

pub enum StatusCode {
    Ok,
    EOF,
    NoSuchFile,
    PermissionDenied,
    Failure,
    BadMessage,
    NoConnection,
    ConnectionLost,
    OperationUnsupported,
}
