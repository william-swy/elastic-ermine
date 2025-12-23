use elastic_ermine::cli;

fn main() {
    let client = cli::create_client();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            println!("Failed to create runtime: {}", e);
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

    match rt.block_on(client.get_indicies()) {
        Ok(indicies) => {
            println!("Indicies:");
            for index in indicies.iter() {
                println!("\tname: {}, uuid: {}, docs: {}, deleted docs: {}, size: {}, primary shards: {}, replica shards: {}", 
                    index.name, index.uuid, 
                    index.docs_count.as_ref().map(String::as_str).unwrap_or("Unknown"), 
                    index.docs_deleted_count.as_ref().map(String::as_str).unwrap_or("Unknown"), 
                    index.dataset_size.as_ref().map(String::as_str).unwrap_or("Unknown"),
                    index.primary_shard_count,
                    index.replica_shard_count,
                );
            }
        },
        Err(err) => {
            println!("Failed to get indicies: {}", err);
            std::process::exit(1);
        },
    }


}