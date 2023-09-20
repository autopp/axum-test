use ::anyhow::Result;
use ::reserve_port::ReservedPort;
use ::std::net::IpAddr;
use ::std::net::Ipv4Addr;
use ::std::net::SocketAddr;

use crate::TestServer;
use crate::IntoTestServerThread;

pub(crate) const DEFAULT_IP_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug)]
pub struct TestServerBuilder {
  port: Option<TestServerPort>,
  ip: Option<IpAddr>,

  save_cookies: bool,
  expect_success_by_default: bool,
  restrict_requests_with_http_schema: bool,
  default_content_type: Option<String>,
}

impl TestServerBuilder {
  pub fn new() -> Self {
    Self {
      port: None,
      ip: None,
      save_cookies: false,
      expect_success_by_default: false,
      restrict_requests_with_http_schema: false,
      default_content_type: None,
    }
  }

  pub fn build<A>(self, app: A) -> Result<TestServer>
    where
        A: IntoTestServerThread,
  {
    self.register_port_if_missing()?;
    TestServer::new_with_builder(app, self)
  }

  pub fn set_port(mut self, port: u16) -> Self {
    ReservedPort::reserve_port(port)
    self.port = Some(TestServerPort::Named(port));
    self
  }

  pub fn port(&mut self) -> Result<u16> {
    self.register_port_if_missing()
  }

  pub fn set_ip(mut self, ip: IpAddr) -> Self {
    self.ip = Some(ip);
    self
  }

  pub fn ip(&self) -> IpAddr {
    self.ip.unwrap_or(DEFAULT_IP_ADDRESS)
  }

  fn socket_address(&mut self) -> Result<SocketAddr> {
    let port = self.port()?;
    let ip = self.ip();
    let socket_addr = SocketAddr::new(ip, port);

    Ok(socket_addr)
  }

  /// Returns what the local address for this server will be when it is created.
  ///
  /// By default this will be something like `http://0.0.0.0:1234/`,
  /// where `1234` is a randomly assigned port numbr.
  pub fn server_address(&mut self) -> Result<String> {
    let socket_addr = self.socket_address()?;
    let server_addr = format!("http://{socket_addr}");

    Ok(server_addr)
  }

  fn register_port_if_missing(&mut self) -> Result<u16> {
    match &self.port {
      None => {
        let reserved_port = ReservedPort::random()?;
        let port = reserved_port.port();
        self.port = Some(TestServerPort::Reserved(reserved_port));

        Ok(port)
      },
      Some(TestServerPort::Reserved(reserve_port)) => Ok(reserve_port.port()),
      Some(TestServerPort::Named(port)) => Ok(*port),
    }
  }
}

#[derive(Debug)]
pub enum TestServerPort {
  Named(u16),
  Reserved(ReservedPort),
}
