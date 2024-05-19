/// HTTP Method
#[derive(Clone, Eq, Copy, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
    Head,
    Patch,
    Options,
}

impl Method {
    pub fn from_str(val: &str) -> Result<Self, ()> {
        match val {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "DELETE" => Ok(Self::Delete),
            "PUT" => Ok(Self::Put),
            "HEAD" => Ok(Self::Head),
            "PATCH" => Ok(Self::Patch),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Put => "PUT",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
        }
    }

    pub fn has_body(&self) -> bool {
        match self {
            Self::Post | Self::Put | Self::Patch => true,
            _ => false,
        }
    }
}
