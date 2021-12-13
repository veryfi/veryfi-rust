use hmac_sha256::HMAC;
#[cfg(test)]
use mockito::server_url;
use reqwest::blocking::Client as Session;
use reqwest::Method;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use utf8_decode::Decoder;

/// Veryfi Rust Client to call Veryfi API.
pub struct Client {
    /// Client id provided by Veryfi.
    pub client_id: String,
    /// Client secret provided by Veryfi.
    pub client_secret: String,
    /// Username provided by Veryfi
    pub username: String,
    /// Api key provided by Veryfi.
    pub api_key: String,
    /// Base URL to Veryfi API.
    pub url: String,
    /// Api Version
    pub api_version: String,
    /// Api timeout to call Veryfi API by default 120.
    pub api_timeout: Duration,
}

impl Client {
    fn get_url(&self) -> String {
        #[cfg(not(test))]
        return format!("{}{}/partner", self.url, self.api_version);
        #[cfg(test)]
        return server_url();
    }

    /// Generate unique signature for payload params.
    /// # Arguments
    ///
    /// * `payload_params` - Map object with payload params to send to Veryfi.
    /// * `timestamp` - timestamp to generate signature.
    ///
    /// * `return` - A String with unique signature generated using the client_secret and the payload.
    fn generate_signature(&self, payload_params: &Map<String, Value>, timestamp: u128) -> String {
        let mut payload: String = format!("timestamp:{}", timestamp);
        for (key, value) in payload_params.clone() {
            payload = format!("{},{}:{}", payload, key, value);
        }
        let const_payload: String = payload;
        let payload_bytes = const_payload.as_bytes();
        let temporary_signature = HMAC::mac(payload_bytes, self.client_secret.as_bytes());
        let without_encode_signature = base64::encode(temporary_signature);
        let base64_signature = without_encode_signature.as_bytes();
        let decoder = Decoder::new(base64_signature.iter().cloned());
        let mut signature = String::new();
        for character in decoder {
            signature.push(character.unwrap());
        }
        return signature;
    }

    /// Submit the HTTP request.
    /// # Arguments
    ///
    /// * `http_verb` - The Http method.
    /// * `endpoint_name` - Endpoint name such as "documents", "users", etc.
    /// * `request_arguments` - Map object with payload params to send to Veryfi.
    ///
    /// * `return` - A String with the JSON response data.
    fn request(
        &self,
        http_verb: reqwest::Method,
        endpoint_name: String,
        request_arguments: &Map<String, Value>,
    ) -> String {
        let time_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let signature = self.generate_signature(request_arguments, time_stamp);
        let api_url = format!("{}{}", self.get_url(), endpoint_name);
        let request = Session::new()
            .request(http_verb, api_url)
            .header("User-Agent", "Rust Veryfi-Rust/1.0.0")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Client-Id", self.client_id.clone())
            .header(
                "Authorization",
                format!("apikey {}:{}", self.username, self.api_key),
            )
            .header("X-Veryfi-Request-Timestamp", time_stamp.to_string())
            .header("X-Veryfi-Request-Signature", signature)
            .json(request_arguments)
            .timeout(self.api_timeout);
        return request.send().unwrap().text().unwrap();
    }

    /// Get the default categories to process document.
    /// # Arguments
    ///
    /// * `return` - A Vector with default categories
    fn get_categories(&self) -> Vec<&str> {
        return vec![
            "Advertising & Marketing",
            "Automotive",
            "Bank Charges & Fees",
            "Legal & Professional Services",
            "Insurance",
            "Meals & Entertainment",
            "Office Supplies & Software",
            "Taxes & Licenses",
            "Travel",
            "Rent & Lease",
            "Repairs & Maintenance",
            "Payroll",
            "Utilities",
            "Job Supplies",
            "Grocery",
        ];
    }

    /// Get list of documents
    ///
    /// * `return` - A JSON String list of processes documents and metadata.
    pub fn get_documents(&self) -> String {
        let endpoint_name: String = "/documents/".to_string();
        let request_arguments = Map::new();
        return self.request(Method::GET, endpoint_name, &request_arguments);
    }

    /// Retrieve document by ID.
    /// # Arguments
    ///
    /// * `document_id` - ID of the document you'd like to retrieve.
    ///
    /// * `return` - A JSON String of data extracted from the Document.
    pub fn get_document(&self, document_id: String) -> String {
        let endpoint_name = format!("/documents/{}/", document_id);
        let mut request_arguments = Map::new();
        request_arguments.insert("id".to_string(), Value::from(&*document_id));
        return self.request(Method::GET, endpoint_name, &request_arguments);
    }

    /// Process a document and extract all the fields from it.
    /// # Arguments
    ///
    /// * `file_path` - Path on disk to a file to submit for data extraction.
    /// * `categories` - Array of categories Veryfi can use to categorize the document.
    /// * `delete_after_processing` - Delete this document from Veryfi after data has been extracted.
    /// * `additional_request_parameters` - Map with Additional request parameters.
    ///
    /// * `return` - A JSON String with data extracted from the document.
    pub fn process_document(
        &self,
        file_path: &str,
        categories: Vec<&str>,
        delete_after_processing: bool,
        additional_parameters: Map<String, Value>,
    ) -> String {
        let endpoint_name = "/documents/".to_string();
        let path = Path::new(file_path);
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let file = File::open(path).unwrap();
        let mut file_reader = BufReader::new(file);
        let mut image_file = Vec::new();
        file_reader.read_to_end(&mut image_file).unwrap();
        let image_data = base64::encode(image_file);
        let mut request_arguments = Map::new();
        request_arguments.insert("file_name".to_string(), Value::from(file_name));
        request_arguments.insert("file_data".to_string(), Value::from(image_data));
        if categories.is_empty() {
            request_arguments.insert("categories".to_string(), Value::from(self.get_categories()));
        } else {
            request_arguments.insert("categories".to_string(), Value::from(categories));
        }
        request_arguments.insert(
            "auto_delete".to_string(),
            Value::from(delete_after_processing),
        );
        request_arguments.extend(additional_parameters);
        return self.request(Method::POST, endpoint_name, &request_arguments);
    }

    /// Process Document from url and extract all the fields from it.
    /// # Arguments
    ///
    /// * `file_url` - Publicly accessible URL to a file, e.g. "https://cdn.example.com/receipt.jpg".
    /// * `categories` - Array of categories to use when categorizing the document.
    /// * `delete_after_processing` - Delete this/these document(s) from Veryfi after data has been extracted.
    /// * `boost_mode` - Flag that tells Veryfi whether boost mode should be enabled. When set to 1, Veryfi will skip data enrichment steps, but will process the document faster.
    /// * `external_id` - Optional custom document identifier. Use this if you would like to assign your own ID to documents.
    /// * `max_pages_to_process` - When sending a long document to Veryfi for processing, this parameter controls how many pages of the document will be read and processed, starting from page 1.
    /// * `additional_request_parameters` - Additional request parameters.
    ///
    /// * `return` - A JSON String with data extracted from the document.
    pub fn process_document_url(
        &self,
        file_url: &str,
        categories: Vec<&str>,
        delete_after_processing: bool,
        boost_mode: i32,
        external_id: &str,
        max_pages_to_process: i32,
        file_urls: Vec<&str>,
        additional_parameters: Map<String, Value>,
    ) -> String {
        let endpoint_name: String = "/documents/".to_string();
        let mut request_arguments = Map::new();
        request_arguments.insert(
            "auto_delete".to_string(),
            Value::from(delete_after_processing),
        );
        request_arguments.insert("boost_mode".to_string(), Value::from(boost_mode));
        request_arguments.insert("categories".to_string(), Value::from(categories));
        if !external_id.is_empty() {
            request_arguments.insert("external_id".to_string(), Value::from(external_id));
        }
        request_arguments.insert("file_url".to_string(), Value::from(file_url));
        if !file_urls.is_empty() {
            request_arguments.insert("file_urls".to_string(), Value::from(file_urls));
        }
        if max_pages_to_process > 0 {
            request_arguments.insert(
                "max_pages_to_process".to_string(),
                Value::from(max_pages_to_process),
            );
        }
        request_arguments.extend(additional_parameters);
        return self.request(Method::POST, endpoint_name, &request_arguments);
    }

    /// Update data for a previously processed document, including almost any field like `vendor`, `date`, `notes` and etc.
    /// # Arguments
    ///
    /// * `document_id` - ID of the document you"d like to update.
    /// * `parameters_to_update` - Fields to update.
    ///
    /// * `return` - A JSON String  with updated fields, if fields are writable. Otherwise a document with unchanged fields.
    pub fn update_document(
        &self,
        document_id: &str,
        parameters_to_update: Map<String, Value>,
    ) -> String {
        let endpoint_name = format!("/documents/{}/", document_id);
        return self.request(Method::PUT, endpoint_name, &parameters_to_update);
    }

    /// Delete Document from Veryfi.
    /// # Arguments
    ///
    /// * `document_id` - ID of the document you'd like to delete.
    ///
    /// * `return` A JSON String with response code and a message.
    pub fn delete_document(&self, document_id: &str) -> String {
        let endpoint_name = format!("/documents/{}/", document_id);
        let mut request_arguments = Map::new();
        request_arguments.insert("id".to_string(), Value::from(document_id));
        return self.request(Method::DELETE, endpoint_name, &request_arguments);
    }
}

#[cfg(test)]
mod tests {
    use crate::client::create_client;
    use mockito::mock;
    use rand::{distributions::Alphanumeric, Rng};
    use serde_json::{from_str, Map, Value};
    use std::path::PathBuf;

    static CLIENT_ID: &str = "client_id";
    static CLIENT_SECRET: &str = "client_secret";
    static USERNAME: &str = "username";
    static API_KEY: &str = "api_key";

    #[test]
    fn test_get_documents() {
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let mock = mock("GET", "/documents/")
            .with_body_from_file(&*format!(
                "{}/resources/getDocuments.json",
                get_cargo_path()
            ))
            .create();
        let response = client.get_documents();
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let json_documents = json_response.get("documents").unwrap();
        assert!(json_documents.is_array());
    }

    #[test]
    fn test_get_document() {
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let id = "23213".to_string();
        let path = &*format!("/documents/{}/", id);
        let mock = mock("GET", path)
            .with_body_from_file(&*format!("{}/resources/getDocument.json", get_cargo_path()))
            .create();
        let response = client.get_document(id);
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let id = json_response.get("id").unwrap();
        assert!(id.is_number());
    }

    #[test]
    fn test_process_document() {
        let file_path = &*format!("{}/resources/receipt.jpeg", get_cargo_path());
        let categories = Vec::new();
        let additional_parameters = Map::new();
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let mock = mock("POST", "/documents/")
            .with_body_from_file(&*format!(
                "{}/resources/processDocument.json",
                get_cargo_path()
            ))
            .create();
        let response = client.process_document(file_path, categories, true, additional_parameters);
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let id = json_response.get("id").unwrap();
        assert!(id.is_number());
    }

    #[test]
    fn test_process_document_url() {
        let categories = Vec::new();
        let additional_parameters = Map::new();
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let url = "https://veryfi-testing-public.s3.us-west-2.amazonaws.com/receipt.jpg";
        let mock = mock("POST", "/documents/")
            .with_body_from_file(&*format!(
                "{}/resources/processDocument.json",
                get_cargo_path()
            ))
            .create();
        let response = client.process_document_url(
            url,
            categories,
            true,
            1,
            "",
            1,
            Vec::new(),
            additional_parameters,
        );
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let id = json_response.get("id").unwrap();
        assert!(id.is_number());
    }

    #[test]
    fn test_update_document() {
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let notes = generate_random_string();
        let id = "49089556";
        let mut parameters = Map::new();
        parameters.insert("notes".to_string(), Value::from(notes.clone()));
        let path = &*format!("/documents/{}/", id);
        let mock = mock("PUT", path)
            .with_body_from_file(&*format!(
                "{}/resources/updateDocument.json",
                get_cargo_path()
            ))
            .create();
        let response = client.update_document(id, parameters);
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let id = json_response.get("id").unwrap();
        assert!(id.is_number());
        let response_notes = json_response.get("notes").unwrap();
        assert!(response_notes.is_string());
        assert_eq!(response_notes.as_str().unwrap().to_string().len(), 10);
    }

    #[test]
    fn test_delete_document() {
        let client = create_client(
            CLIENT_ID.to_string().clone(),
            CLIENT_SECRET.to_string().clone(),
            USERNAME.to_string().clone(),
            API_KEY.to_string().clone(),
        );
        let id = "35365";
        let path = &*format!("/documents/{}/", id);
        let mock = mock("DELETE", path)
            .with_body_from_file(&*format!(
                "{}/resources/deleteDocument.json",
                get_cargo_path()
            ))
            .create();
        let response = client.delete_document(&*id);
        mock.assert();
        let json_response: Value = from_str(&*response).unwrap();
        assert!(json_response.is_object());
        let message = json_response.get("message").unwrap();
        assert!(message.is_string());
        assert_eq!(message.as_str().unwrap(), "Document has been deleted");
        let status = json_response.get("status").unwrap();
        assert!(status.is_string());
        assert_eq!(status.as_str().unwrap(), "ok");
    }

    fn get_cargo_path() -> String {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .to_str()
            .unwrap()
            .to_string();
    }

    fn generate_random_string() -> String {
        return rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
    }
}
