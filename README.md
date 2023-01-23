https://docs.rs/veryfi/1.0.0/veryfi/

<img src="https://user-images.githubusercontent.com/30125790/212157461-58bdc714-2f89-44c2-8e4d-d42bee74854e.png#gh-dark-mode-only" width="200">
<img src="https://user-images.githubusercontent.com/30125790/212157486-bfd08c5d-9337-4b78-be6f-230dc63838ba.png#gh-light-mode-only" width="200">

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![code coverage](.github/reports/badges/plastic.svg)](.github/reports/badges/plastic.svg)

**Veryfi** is a Rust module for communicating with the [Veryfi OCR API](https://veryfi.com/api/)

## Installation
Add to your cargo.tml
```toml
veryfi = "1.0.0"
```

## Getting Started

### Obtaining Client ID and user keys
If you don't have an account with Veryfi, please go ahead and register here: [https://hub.veryfi.com/signup/api/](https://hub.veryfi.com/signup/api/)

### Rust API Client Library
The **Veryfi** library can be used to communicate with Veryfi API. All available functionality is described here DOC

Below is the sample script using **Veryfi** to OCR and process data from a document:

### Process a document.

```rust
use veryfi::client::create_client;
use serde_json::{from_str, Map, Value};

fn main() {
    let client_id = "your_client_id".to_string();
    let client_secret = "your_client_secret".to_string();
    let username = "your_username".to_string();
    let api_key = "your_api_key".to_string();
    
    let client = create_client(client_id, client_secret, username, api_key);
    let categories = vec!["Advertising & Marketing", "Automotive"];
    let file_path = "path_to_your_file";
    let delete_after_processing = true;
    let additional_parameters = Map::new();
    
    let response = client.process_document(file_path, categories, delete_after_processing, additional_parameters);
    print!("{}", response); // to print
    let json_response: Value = from_str(&*response).unwrap();
    // ...
}
```

### Update a document

```rust
use veryfi::client::create_client;
use serde_json::{from_str, Map, Value};

fn main() {
    let client_id = "your_client_id".to_string();
    let client_secret = "your_client_secret".to_string();
    let username = "your_username".to_string();
    let api_key = "your_api_key".to_string();

    let client = create_client(client_id, client_secret, username, api_key);
    let document_id = "your_document_id".to_string();
    let mut parameters = Map::new();
    let notes = "your_notes";
    parameters.insert("notes".to_string(), Value::from(notes.clone()));

    let response = client.update_document(document_id, parameters);
    print!("{}", response); // to print
    let json_response: Value = from_str(&*response).unwrap();
    // ...
}
```

## Need help?
If you run into any issue or need help installing or using the library, please contact support@veryfi.com.

If you found a bug in this library or would like new features added, then open an issue or pull requests against this repo!

To learn more about Veryfi visit https://www.veryfi.com/

## Tutorial


Below is an introduction to the Rust SDK.
