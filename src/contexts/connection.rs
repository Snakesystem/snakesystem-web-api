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

    pub async fn rollback(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn_guard = self.conn.lock().await;
        if let Some(mut conn) = conn_guard.take() {
            conn.simple_query("ROLLBACK").await?;
        }
        self.committed = false;
        Ok(())
    }

    pub async fn begin_transaction<T, F, Fut>(
        pool: &Pool<ConnectionManager>,
        f: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(&mut PooledConnection<ConnectionManager>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
        T: Send + 'static,
    {
        let mut conn = pool.get().await?;
        conn.simple_query("BEGIN").await?;

        let result = f(&mut conn).await;

        match result {
            Ok(val) => {
                conn.simple_query("COMMIT").await?;
                Ok(val)
            }
            Err(e) => {
                let _ = conn.simple_query("ROLLBACK").await;
                Err(e)
            }
        }
    }

}

// impl<'a> Drop for Transaction<'a> {
//     fn drop(&mut self) {
//         // Kalau belum commit, rollback (sync!)
//         if !self.committed {
//             // WARNING: ini blocking dan sync, jadi gak cocok untuk async
//             let conn = self.conn.blocking_lock().take();
//             if let Some(mut conn) = conn {
//                 // rollback sync — pakai try block biar gak panik
//                 println!("Rollback transaction");
//                 let _ = conn.simple_query("ROLLBACK");
//             }
//         }
//     }
// }
impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // gak bisa `.await` di Drop, jadi lebih aman kalau commit manual
            eprintln!("Transaction dropped without commit, should rollback.");
        }
    }
}

/// Membuat pool koneksi database
pub async fn create_pool(db_server: &str, db_user: &str, db_password: &str, database: &str) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {

    let connection_string = format!(
        "Server={};User={};Password={};TrustServerCertificate=true;Database={}",
        db_server, db_user, db_password, database
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