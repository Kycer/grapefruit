
pub struct TransactionManager<'a> {
    pool: &'a PgPool,
}

impl<'a> TransactionManager<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        TransactionManager { pool }
    }

    pub async fn begin(&self) -> sqlx::Result<Transaction<'a, sqlx::Postgres>> {
        self.pool.begin().await
    }

    pub async fn with_transaction<T, F>(&self, f: F) -> sqlx::Result<T>
    where
        F: FnOnce(Transaction<'a, sqlx::Postgres>) -> sqlx::Result<T>,
    {
        let mut transaction = self.pool.begin().await?;

        let result = f(transaction.transaction()).await;

        if result.is_ok() {
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }

        result
    }
}

impl<'a> Deref for TransactionManager<'a> {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        self.pool
    }
}

impl<'a> DerefMut for TransactionManager<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.pool
    }
}