#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsError {
    Enoent,
    Enotdir,
    Eisdir,
    Einval,
    Ebusy,
    Erofs,
    Eio,
    Enomem,
    Enodev,
    Espipe,
    Eacces,
    Eagain,
}

pub type FsResult<T> = Result<T, FsError>;

impl FsError {
    pub fn as_isize(&self) -> isize {
        -match self {
            Self::Enoent => 2,
            Self::Enotdir => 20,
            Self::Eisdir => 21,
            Self::Einval => 22,
            Self::Ebusy => 16,
            Self::Erofs => 30,
            Self::Eio => 5,
            Self::Enomem => 12,
            Self::Enodev => 19,
            Self::Espipe => 29,
            Self::Eacces => 13,
            Self::Eagain => 11,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Enoent => "ENOENT",
            Self::Enotdir => "ENOTDIR",
            Self::Eisdir => "EISDIR",
            Self::Einval => "EINVAL",
            Self::Ebusy => "EBUSY",
            Self::Erofs => "EROFS",
            Self::Eio => "EIO",
            Self::Enomem => "ENOMEM",
            Self::Enodev => "ENODEV",
            Self::Espipe => "ESPIPE",
            Self::Eacces => "EACCES",
            Self::Eagain => "EAGAIN",
        }
    }
}

impl core::fmt::Display for FsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::error::Error for FsError {}
