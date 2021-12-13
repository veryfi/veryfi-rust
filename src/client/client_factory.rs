use crate::client::Client;
use std::time::Duration;

/// Create a client with default api key (v8).
/// # Arguments
///
/// * `client_id` - Client id provided by Veryfi.
/// * `client_secret` - Client secret provided by Veryfi.
/// * `username` - Username provided by Veryfi.
/// * `api_key` - Api key provided by Veryfi.
///
/// * `return` - A veryfi::client::Client object to call Veryfi API.
///
/// # Example
///
/// ```
/// use veryfi::client::create_client;
///
/// let client_id = "your_client_id".to_string();
/// let client_secret = "your_client_secret".to_string();
/// let username = "your_username".to_string();
/// let api_key = "your_api_key".to_string();
/// let client = create_client(client_id, client_secret, username, api_key);
/// ```
pub fn create_client(
    client_id: String,
    client_secret: String,
    username: String,
    api_key: String,
) -> Client {
    let api_version = "v8".to_string();
    return create_client_with_custom_api_version(
        client_id,
        client_secret,
        username,
        api_key,
        api_version,
    );
}

/// Create a client with custom api key.
/// # Arguments
///
/// * `client_id` - Client id provided by Veryfi.
/// * `client_secret` - Client secret provided by Veryfi.
/// * `username` - Username provided by Veryfi.
/// * `api_key` - Api key provided by Veryfi.
/// * `api_version` - Api version to use Veryfi.
///
/// * `return` - A veryfi::client::Client object to call Veryfi API.
pub fn create_client_with_custom_api_version(
    client_id: String,
    client_secret: String,
    username: String,
    api_key: String,
    api_version: String,
) -> Client {
    let url = "https://api.veryfi.com/api/".to_string();
    let api_timeout = Duration::from_secs(120);
    let client = Client {
        client_id,
        client_secret,
        username,
        api_key,
        url,
        api_version,
        api_timeout,
    };
    return client;
}
