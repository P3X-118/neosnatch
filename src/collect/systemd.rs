use anyhow::Result;
use zbus::{proxy, Connection};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    fn list_units_by_patterns(
        &self,
        states: Vec<&str>,
        patterns: Vec<&str>,
    ) -> zbus::Result<Vec<UnitTuple>>;
}

type UnitTuple = (
    String,                    // name
    String,                    // description
    String,                    // load_state
    String,                    // active_state
    String,                    // sub_state
    String,                    // following
    zbus::zvariant::OwnedObjectPath,
    u32,
    String,
    zbus::zvariant::OwnedObjectPath,
);

pub async fn failed_units() -> Result<Vec<String>> {
    let conn = match Connection::system().await {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    let proxy = match ManagerProxy::new(&conn).await {
        Ok(p) => p,
        Err(_) => return Ok(Vec::new()),
    };
    let units = match proxy.list_units_by_patterns(vec!["failed"], vec![]).await {
        Ok(u) => u,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(units.into_iter().map(|t| t.0).collect())
}
