use elastic_ermine::{cli,es};

fn main() {
    let client = cli::create_client();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            println!("Failed to create runtime: {}", e);
            std::process::exit(1);
        });

    let body = serde_json::from_str::<serde_json::Value>(
        r#"
        {
            "size": 10,
            "query": {
                "match_all": {}
            }
        }
        "#)
        .unwrap_or_else(|e| {
            println!("Failed to build request body: {}", e);
            std::process::exit(1);
        });

    match rt.block_on(client.test_connection()) {
        Ok(_) => {
            println!("Test connection success");
        },
        Err(err) => {
            println!("Test connection failed: {}", err);
            std::process::exit(1);
        },
    };

    match rt.block_on(client.operation(
        es::ElasticSearchMethodType::POST, 
        "/*/_search", Some(&body))) {
        Ok(res) => {
            println!("Get document result\n {:?}", res);
        },
        Err(err) => {
            println!("Get documents failed: {}", err);
            std::process::exit(1);
        },
    }
}