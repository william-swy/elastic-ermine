pub enum Auth {
    BASIC(BasicAuth),
}

pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>
}

pub struct ElasticsearchClient {
    config: ClientConfig,
    client: reqwest::Client,
}

struct ClientConfig {
    root_url: String,
    auth: Option<Auth>,
    cert: Option<reqwest::Certificate>,
}

impl ClientConfig {
    fn build_reqwest_client(&self) -> Result<reqwest::Client, reqwest::Error> {
        let mut builder = reqwest::Client::builder().use_rustls_tls();

        if let Some(cert) = &self.cert {
            builder = builder.add_root_certificate(cert.clone());
        }

        return builder.build();
    }
}

#[derive(Debug, Clone)]
struct ElasticSearchError {
    err: String,
}

impl ElasticSearchError {
    fn new(err: String) -> Self {
        return Self {
            err: err,
        }
    }
}

impl std::fmt::Display for ElasticSearchError {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(f, "{}", self.err);
    }
}

impl std::error::Error for ElasticSearchError {}

#[derive(Clone)]
pub struct ElasticSearchIndex {
    pub name: String,
    pub uuid: String,
    pub primary_shard_count: String,
    pub replica_shard_count: String,
    pub docs_count: Option<String>,
    pub docs_deleted_count: Option<String>,
    pub dataset_size: Option<String>,
}

pub enum ElasticSearchMethodType {
    POST,
}

impl ElasticsearchClient {
    pub fn new(root_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config = ClientConfig {
            root_url: root_url,
            auth: None,
            cert: None,
        };

        let client = config.build_reqwest_client()?;

        return Ok(Self {
            config: config,
            client: client,
        })
    }

    pub fn use_auth(&mut self, auth: Auth) {
        self.config.auth = Some(auth);
    }

    // True by default
    pub fn use_no_auth(&mut self) {
        self.config.auth = None;
    }

    pub fn use_custom_pem_certificate<P: AsRef<std::path::Path>>(&mut self, cert_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let data = std::fs::read(cert_path)?;

        let cert = reqwest::Certificate::from_pem(&data)?;

        self.config.cert = Some(cert);

        self.client = self.config.build_reqwest_client()?;

        return Ok(());
    }

    // True by default
    pub fn use_default_certificates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.cert = None;

        self.client = self.config.build_reqwest_client()?;

        return Ok(());
    }

    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = reqwest::Url::parse(&self.config.root_url)?;
        let builder = self.client.get(url);

        let builder = self.request_add_auth(builder);

        builder.send().await?.error_for_status()?;

        return Ok(());
    }

    fn request_add_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(auth) = &self.config.auth {
            match auth {
                Auth::BASIC(basic_auth) => {
                    return request.basic_auth(&basic_auth.username, basic_auth.password.clone());
                },
            }
        }
        return request;
    }

    pub async fn get_indicies(&self) -> Result<Vec<ElasticSearchIndex>, Box<dyn std::error::Error>> {
        let base_url = reqwest::Url::parse(&self.config.root_url)?;
        let url = base_url.join("_cat/indices?format=json")?;

        let builder = self.client.get(url);

        let builder = self.request_add_auth(builder);

        let res = builder.send().await?.text().await?;

        let json: serde_json::Value = serde_json::from_str(&res)?;

        let res = json.as_array()
            .ok_or(ElasticSearchError::new("Response was not an array".to_owned()))?
            .iter()
            .map(ElasticsearchClient::parse_indicices_response)
        .collect::<Result<Vec<_>, _>>()?;

        return Ok(res);
    }

    pub async fn operation(&self, method_type: ElasticSearchMethodType, path: &str, body: Option<&serde_json:: Value>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let base_url = reqwest::Url::parse(&self.config.root_url)?;
        let url = base_url.join(path)?;

        let mut builder = match method_type {
            ElasticSearchMethodType::POST => self.client.post(url),
        };

        builder = self.request_add_auth(builder);

        if let Some(request_body) = body {
            builder = builder.json(request_body);
        }

        let res = builder.send().await?.text().await?;

        let json = serde_json::from_str(&res)?;

        return Ok(json);
    }

    fn parse_indicices_response(val: &serde_json::Value) -> Result<ElasticSearchIndex, Box<dyn std::error::Error>> {
        let name = val["index"]
            .as_str()
            .ok_or(
                ElasticSearchError::new(
                    format!("index entry was not a valid string: {}", val["index"]).to_owned()
                )
            )?
            .to_owned();

        let uuid = val["uuid"]
            .as_str()
            .ok_or(
                ElasticSearchError::new(
                    format!("uuid entry is not in expected string format: {}", val["uuid"]).to_owned()
                )
            )?
            .to_owned();
        let primary_shard_count = val["pri"]
            .as_str()
            .ok_or(
                ElasticSearchError::new(
                    format!("pri entry is not in expected string format: {}", val["pri"]).to_owned()
                )
            )?
            .to_owned();
        let replica_shard_count = val["rep"]
            .as_str()
            .ok_or(
                ElasticSearchError::new(
                    format!("rep entry is not in expected string format: {}", val["rep"]).to_owned()
                )
            )?
            .to_owned();
        let docs_count = match &val["docs.count"] {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(count_str) => Ok(Some(count_str.to_owned())),
            _ => Err(ElasticSearchError::new(
                format!("docs.count entry is not null or a string: {}", val["docs.count"])
            )),
        }?;

        let docs_deleted_count = match &val["docs.deleted"] {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(count_str) => Ok(Some(count_str.to_owned())),
            _ => Err(ElasticSearchError::new(
                format!("docs.deleted entry is not null or a string: {}", val["docs.deleted"])
            )),
        }?;

        let dataset_size = match &val["dataset.size"] {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(count_str) => Ok(Some(count_str.to_owned())),
            _ => Err(ElasticSearchError::new(
                format!("dataset.size entry is not null or a string: {}", val["dataset.size"])
            )),
        }?;

        return Ok(
            ElasticSearchIndex { 
                name, 
                uuid, 
                primary_shard_count, 
                replica_shard_count, 
                docs_count, 
                docs_deleted_count, 
                dataset_size
            }
        );
    }
}