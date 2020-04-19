use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: usize,
    auth: Option<String>,
}

impl Request {
    pub fn create_login_request(
        user: &str,
        password: &str,
        nonce: Option<usize>,
    ) -> Result<Request> {
        let req = Request {
            jsonrpc: String::from("2.0"),
            method: String::from("user.login"),
            params: json!({
                "user": String::from(user),
                "password": String::from(password)
            }),
            id: nonce.unwrap(),
            auth: None,
        };

        Ok(req)
    }

    pub fn create_request(
        method: &str,
        params: &serde_json::Value,
        auth: &str,
        nonce: &usize,
    ) -> Result<Request> {
        let req = Request {
            jsonrpc: String::from("2.0"),
            method: method.to_string(),
            params: params.clone(),
            id: *nonce,
            auth: Some(auth.to_string()),
        };

        Ok(req)
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    jsonrpc: String,
    error: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    id: usize,
}

impl Response {
    pub fn auth(&self) -> Result<String> {
        let result = self.result.as_ref().unwrap();
        let auth = result.as_str().unwrap();

        Ok(auth.to_string())
    }
}

#[derive(Debug)]
struct ZabbixApi {
    url: String,
    auth: String,
    client: reqwest::Client,
    nonce: usize,
}

impl ZabbixApi {
    pub async fn new(url: &str, user: &str, password: &str) -> Result<ZabbixApi> {
        let client = reqwest::Client::new();
        let nonce = 1;
        let req = Request::create_login_request(user, password, Some(nonce)).unwrap();

        let res = client
            .post(url)
            .json(&req)
            .send()
            .await?
            .json::<Response>()
            .await?;

        let auth = res.auth()?;

        let api = ZabbixApi {
            url: String::from(url),
            client: client,
            nonce: nonce,
            auth: auth,
        };

        Ok(api)
    }

    pub async fn host_get(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let req =
            Request::create_request("host.get", params, self.auth.as_str(), &self.nonce).unwrap();
        let res = self
            .client
            .post(self.url.as_str())
            .json(&req)
            .send()
            .await?
            .json::<Response>()
            .await?;

        Ok(res.result.unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://localhost:8080/api_jsonrpc.php";
    let user = "api";
    let password = "0qww294e";

    let api = ZabbixApi::new(url, user, password).await?;

    println!("{:#?}", api);

    let params = json!({});
    let hosts = api.host_get(&params).await?;

    println!("{:#?}", hosts);

    Ok(())
}
