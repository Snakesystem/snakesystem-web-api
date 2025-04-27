use bb8::{Pool, PooledConnection};
use bb8_tiberius::ConnectionManager;
use tiberius::Config;
use tokio::sync::{Mutex, MutexGuard};
use std::sync::Arc;

pub type DbPool = Pool<ConnectionManager>;
pub struct Transaction<'a> {
    pub conn: Arc<Mutex<Option<PooledConnection<'a, ConnectionManager>>>>, // 🔥 Pakai lifetime 'a
    committed: bool,
}

impl<'a> Transaction<'a> {
    pub async fn begin(pool: &'a Pool<ConnectionManager>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = pool.get().await?; // ✅ Tidak pakai 'static, langsung gunakan 'a
        conn.simple_query("BEGIN TRANSACTION").await?; // Mulai transaksi

        Ok(Self {
            conn: Arc::new(Mutex::new(Some(conn))),
            committed: false,
        })
    }

    pub async fn commit(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn_guard: MutexGuard<Option<PooledConnection<ConnectionManager>>> = self.conn.lock().await;
        if let Some(mut conn) = conn_guard.take() {
            conn.simple_query("COMMIT").await?;
        }
        self.committed = true;
        Ok(())
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            if let Ok(mut conn_guard) = self.conn.try_lock() {
                if let Some(mut conn) = conn_guard.take() {
                    // Eksekusi rollback langsung tanpa async
                    let _ = conn.simple_query("ROLLBACK");
                }
            }
        }
    }
}

/// Membuat pool koneksi database
pub async fn create_pool(database: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {

    let connection_string = format!(
        "Server=tcp:db12877.public.databaseasp.net;User=db12877;Password=Snakesystem@09;TrustServerCertificate=true;Database={}",
        database
    );

    let config: Config = Config::from_ado_string(&connection_string)?;
    let manager: ConnectionManager = ConnectionManager::new(config);
    let pool: Pool<ConnectionManager> = Pool::builder()
            .max_size(10)
            .connection_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(60))
            .max_lifetime(std::time::Duration::from_secs(300))
            .max_size(10)
            .build(manager).await?;

    Ok(pool)
}