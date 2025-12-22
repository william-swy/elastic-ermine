use elastic_ermine::es;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("args = {:?}", args);

    let mut url_arg: Option<String> = None;
    let mut auth_arg: Option<String> = None;
    let mut cacert_arg: Option<String> = None;

    let mut idx = 1;

    while idx < args.len() {
        if args[idx] == "--url" {
            url_arg = Some(args[idx+1].clone());
            idx +=2;

        } else if args[idx] == "--auth" {
            // In form of "<username>:<password>"
            // assumes username and password does not contain the character ':'
            auth_arg = Some(args[idx+1].clone());
            idx += 2;
            
        } else if args[idx] == "--cacert" {
            cacert_arg = Some(args[idx+1].clone());
            idx += 2;
        } else {
            println!("Unknown argument {}", args[idx]);
            std::process::exit(1);
        };
    }

    let url = url_arg.unwrap_or_else(|| {
        println!("Missing --url");
        std::process::exit(1);
    });

    let mut client = es::ElasticsearchClient::new(url).unwrap_or_else(|e| {
        println!("Failed to create client: {}", e);
        std::process::exit(1);
    });

    if let Some(auth) = auth_arg {
        let auth_parsed: Vec<&str> = auth.split(":").collect();
        let username = auth_parsed[0];
        let password = auth_parsed.get(1);

        let basic_auth = es::Auth::BASIC(es::BasicAuth{
            username: username.to_string(),
            password: password.map(|x| {x.to_string()})
        });

        client.use_auth(basic_auth);
    }

    if let Some(cacert_file) = cacert_arg {
        client.use_custom_pem_certificate(cacert_file).unwrap_or_else(|e| {
            println!("Failed to create client: {}", e);
            std::process::exit(1);
        });
    }

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
            println!("Get document result\n {}", res);
        },
        Err(err) => {
            println!("Get documents failed: {}", err);
            std::process::exit(1);
        },
    }
}