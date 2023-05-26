// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)] // For nightly lints
#![allow(clippy::uninlined_format_args)]

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let body = reqwest::get("https://jsonplaceholder.typicode.com/posts/1")
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("body = {:#?}", body);

    Ok(())
}
