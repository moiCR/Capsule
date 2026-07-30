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

    if let Ok(conn) = Connection::session().await {
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

    if let Ok(conn) = Connection::system().await {
        if let Ok(mut guard) = SYSTEM_CONN.lock() {
            *guard = Some(conn.clone());
        }
        return Some(conn);
    }

    None
}
