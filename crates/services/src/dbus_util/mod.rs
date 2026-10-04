use std::sync::Mutex;
use zbus::Connection;

static SESSION_CONN: Mutex<Option<Connection>> = Mutex::new(None);
static SYSTEM_CONN: Mutex<Option<Connection>> = Mutex::new(None);

/// Returns a shared, persistent D-Bus Session connection.
/// Automatically reconnects if the cached connection was closed or dropped.
pub async fn get_shared_session_conn() -> Option<Connection> {
    if let Ok(guard) = SESSION_CONN.lock() {
        if let Some(ref conn) = *guard {
            if !conn.is_closed() {
                return Some(conn.clone());
            }
        }
    }

    let conn = if tokio::runtime::Handle::try_current().is_ok() {
        Connection::session().await.ok()
    } else {
        crate::tokio_handle()
            .spawn(async { Connection::session().await.ok() })
            .await
            .ok()
            .flatten()
    };

    if let Some(conn) = conn {
        if let Ok(mut guard) = SESSION_CONN.lock() {
            *guard = Some(conn.clone());
        }
        return Some(conn);
    }

    None
}

/// Returns a shared, persistent D-Bus System connection.
/// Automatically reconnects if the cached connection was closed or dropped.
pub async fn get_shared_system_conn() -> Option<Connection> {
    if let Ok(guard) = SYSTEM_CONN.lock() {
        if let Some(ref conn) = *guard {
            if !conn.is_closed() {
                return Some(conn.clone());
            }
        }
    }

    let conn = if tokio::runtime::Handle::try_current().is_ok() {
        Connection::system().await.ok()
    } else {
        crate::tokio_handle()
            .spawn(async { Connection::system().await.ok() })
            .await
            .ok()
            .flatten()
    };

    if let Some(conn) = conn {
        if let Ok(mut guard) = SYSTEM_CONN.lock() {
            *guard = Some(conn.clone());
        }
        return Some(conn);
    }

    None
}

