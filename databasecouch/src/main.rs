use couch_rs::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("http://127.0.0.1:5984", "admin", "test").unwrap();
    client.make_db("users").await.unwrap();
}
