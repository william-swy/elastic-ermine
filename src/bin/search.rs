use elastic_ermine::cli;

fn main() {
    let client = cli::create_client();

    let mut indicies = Vec::<String>::new();
    let mut params_path: Option<std::path::PathBuf> = None;

    let args: Vec<String> = std::env::args().collect();

    let mut idx = 1;

    while idx < args.len() {
        if args[idx] == "--indicies" {
            indicies = args[idx+1].split(",").map(String::from).collect();
            idx +=2;
        }
        else if args[idx] == "--params" {
            params_path = Some(args[idx+1].to_owned().into());
            idx +=2;
        } else {
            idx += 1;
        }
    }

    let params = params_path.map(|path| {
        let contents = std::fs::read_to_string(&path)
            .expect(&format!("Unable to read {}", path.to_string_lossy()));

        serde_json::from_str::<serde_json::Value>(&contents)
            .expect(&format!("Unable to parse contents of {}", path.to_string_lossy()))
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            println!("Failed to create runtime: {}", e);
            std::process::exit(1);
        });

    println!("Invoking search with indicies: {:?}, body: {:?}", &indicies, &params);

    match rt.block_on(client.search(&indicies, params.as_ref())) {
        Ok(res) => {
            println!("{:?}", res);
        },
        Err(err) => {
            println!("search failed: {}", err);
            std::process::exit(1);
        },
    };

}