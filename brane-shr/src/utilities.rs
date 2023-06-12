//  UTILITIES.rs
//    by Lut99
// 
//  Created:
//    18 Aug 2022, 14:58:16
//  Last edited:
//    12 Jun 2023, 13:46:37
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines common utilities across the Brane project.
// 

use log::{debug, warn};
use regex::Regex;
use url::{Host, Url};


/***** TESTS *****/
#[cfg(test)]
mod tests {
    use super::*;

    /// Test some basic HTTP schemas
    #[test]
    fn ensurehttpschema_noschema_added() {
        let url = ensure_http_schema("localhost", true).unwrap();
        assert_eq!(url, "https://localhost");

        let url = ensure_http_schema("localhost", false).unwrap();
        assert_eq!(url, "http://localhost");
    }

    /// Test some more basic HTTP schemas
    #[test]
    fn ensurehttpschema_schema_nothing() {
        let url = ensure_http_schema("http://localhost", true).unwrap();
        assert_eq!(url, "http://localhost");

        let url = ensure_http_schema("https://localhost", false).unwrap();
        assert_eq!(url, "https://localhost");
    }
}





/***** ADDRESS CHECKING *****/
///
///
///
pub fn ensure_http_schema<S>(
    url: S,
    secure: bool,
) -> Result<String, url::ParseError>
where
    S: Into<String>,
{
    let url = url.into();
    let re = Regex::new(r"^https?://.*").unwrap();

    let url = if re.is_match(&url) {
        url
    } else {
        format!("{}://{}", if secure { "https" } else { "http" }, url)
    };

    // Check if url is valid.
    let _ = Url::parse(&url)?;

    Ok(url)
}



/// Returns whether the given address is an IP address or not.
/// 
/// The address can already involve paths or an HTTP schema. In that case, only the 'host' part is checked.
/// 
/// Both IPv4 and IPv6 addresses are matched.
/// 
/// # Arguments
/// - `address`: The address to check.
/// 
/// # Returns
/// true if the address is an IP-address, or false otherwise.
pub fn is_ip_addr(address: impl AsRef<str>) -> bool {
    let address: &str = address.as_ref();

    // Attempt to parse with the URL thing
    let url: Url = match Url::parse(address) {
        Ok(url) => url,
        Err(err) => {
            warn!("Given URL '{}' is not a valid URL to begin with: {}", address, err);
            return false;
        },
    };

    // Examine the base
    if let Some(host) = url.host() {
        let res: bool = matches!(host, Host::Ipv4(_) | Host::Ipv6(_));
        debug!("Address '{}' has a{} as hostname", address, if res { "n IP address" } else { " domain" });
        matches!(host, Host::Ipv4(_) | Host::Ipv6(_))
    } else {
        debug!("Address '{}' has no hostname (so also no IP address)", address);
        false
    }
}
