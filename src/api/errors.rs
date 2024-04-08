use std::error::Error;

// Custom Errors
#[derive(Debug)]
pub struct GitHubBaseUrlUndefined;
impl std::fmt::Display for GitHubBaseUrlUndefined {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GitHub base URL is undefined")
    }
}
impl Error for GitHubBaseUrlUndefined {}

#[derive(Debug)]
pub struct GitHubTokenUndefined;
impl std::fmt::Display for GitHubTokenUndefined {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GitHub token is undefined")
    }
}
impl Error for GitHubTokenUndefined {}
