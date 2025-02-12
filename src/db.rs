use std::net::Ipv4Addr;

#[derive(Clone, Copy)]
pub struct DB {}
impl DB {
  pub async fn get_country(&self, _: Ipv4Addr) -> String {
    tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
    "USA".to_owned()
  }
}
