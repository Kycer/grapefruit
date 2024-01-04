use std::sync::Arc;

use once_cell::sync::{Lazy, OnceCell};

use crate::{
    build_delete_sql, build_insert_sql, build_logic_delete, build_select_sql, build_update_sql,
    Column, GrapefruitError, GrapefruitOptions, GrapefruitRepository, GrapefruitResult, Page,
    Platform, PlatformPool, Value, Wrapper,
};

pub static GRAPEFRUIT: Lazy<OnceCell<Grapefruit>> = Lazy::new(|| OnceCell::new());

#[derive(Clone)]
pub struct Grapefruit {
    pub(crate) pool: Arc<OnceCell<PlatformPool>>,
    pub(crate) options: GrapefruitOptions,
    pub(crate) platform: Platform,
}

impl Grapefruit {
    pub fn new(config: &GrapefruitOptions) -> Grapefruit {
        Grapefruit {
            pool: Arc::new(OnceCell::new()),
            options: config.clone(),
            platform: config.platform.clone(),
        }
    }

    pub async fn init(&mut self) -> GrapefruitResult<()> {
        let config = &self.options;
        let pool = PlatformPool::new(config).await?;
        self.pool.set(pool).expect("PlatformPool init failed.");
        Ok(())
    }

    pub fn pool(&self) -> &PlatformPool {
        self.pool.get().expect("PlatformPool not init.")
    }

    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub async fn generator_id(&self) -> Value {
        self.options.identifier_generator.next_id().await
    }

    pub fn get_insert_fill(&self, col: &str) -> Result<Value, GrapefruitError> {
        self.options.meta_object.try_get_insert_fill(col)
    }

    pub fn get_update_fill(&self, col: &str) -> Result<Value, GrapefruitError> {
        self.options.meta_object.try_get_update_fill(col)
    }
}

#[async_trait::async_trait]
impl GrapefruitRepository for Grapefruit {
    async fn insert<T>(&self, entity: &T) -> GrapefruitResult<u64>
    where
        T: crate::Entity,
    {
        let (sql, params) = build_insert_sql(&vec![entity], self).await?;

        let row = self.pool().execute(&sql, params).await?;

        Ok(row.rows_affected())
    }

    async fn insert_batch<T>(&self, entities: &[&T]) -> GrapefruitResult<u64>
    where
        T: crate::Entity,
    {
        if entities.is_empty() {
            return Err(GrapefruitError::EmptyEntity);
        }

        let (sql, params) = build_insert_sql(entities, self).await?;

        let row = self.pool().execute(&sql, params).await?;

        Ok(row.rows_affected())
    }

    async fn update_by_id<T>(&self, entity: &T) -> GrapefruitResult<u64>
    where
        T: crate::Entity,
    {
        let primary_key = T::primary_key().alias()?;
        let data = entity.to_value();
        let id = data.get(&primary_key).unwrap();
        if id.is_none() {
            return Err(GrapefruitError::PrimaryKeyNone(
                "Primary key not found when updating entity".to_string(),
            ));
        }

        let (sql, values) = build_update_sql(entity, self, |index| {
            let sql = format!("{} = {}", primary_key, self.platform().mark(index + 1));
            (sql, vec![id.clone()])
        })
        .await?;

        let row = self.pool().execute(&sql, values.into()).await?;

        Ok(row.rows_affected())
    }

    async fn update_by_wrapper<T>(&self, entity: &T, wrapper: Wrapper) -> GrapefruitResult<u64>
    where
        T: crate::Entity,
    {
        let (sql, values) = build_update_sql(entity, self, |index| {
            wrapper.build(self.platform(), index + 1)
        })
        .await?;

        let row = self.pool().execute(&sql, values.into()).await?;

        Ok(row.rows_affected())
    }

    async fn delete_by_id<T, I>(&self, id: I) -> GrapefruitResult<bool>
    where
        T: crate::Entity,
        I: crate::PrimaryKey,
    {
        let primary_key = T::primary_key().alias()?;
        let (sql, params) = build_delete_sql::<T, _>(self, |index| {
            let sql = format!("{} = {}", primary_key, self.platform().mark(index + 1));
            (sql, vec![id.clone().into()])
        })
        .await?;
        let result = self.pool().execute(&sql, params).await?;
        Ok(result.is_success())
    }

    async fn delete_by_ids<T, I>(&self, ids: &[I]) -> GrapefruitResult<bool>
    where
        T: crate::Entity,
        I: crate::PrimaryKey,
    {
        let primary_key = T::primary_key().alias()?;
        let (sql, params) = build_delete_sql::<T, _>(self, |index| {
            let marks = ids
                .iter()
                .enumerate()
                .map(|(i, _)| self.platform().mark(i + index + 1))
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!("{} IN ({})", primary_key, marks);
            (sql, ids.iter().map(|id| id.into()).collect::<Vec<Value>>())
        })
        .await?;
        let result = self.pool().execute(&sql, params).await?;
        Ok(result.is_success())
    }

    async fn delete_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<bool>
    where
        T: crate::Entity,
    {
        let (sql, params) =
            build_delete_sql::<T, _>(self, |index| wrapper.build(self.platform(), index + 1))
                .await?;
        let result = self.pool().execute(&sql, params).await?;
        Ok(result.is_success())
    }

    async fn select_by_id<T, I>(&self, id: I) -> GrapefruitResult<Option<T>>
    where
        T: crate::Entity + crate::TryGetable,
        I: crate::PrimaryKey,
    {
        let primary_key = T::primary_key().alias()?;
        let (sql, params) = build_select_sql::<T, _>(self, |index| {
            let sql = format!("{} = {}", primary_key, self.platform().mark(index + 1));
            (sql, vec![id.clone().into()])
        })
        .await?;
        let query_result = self.pool().fetch_one(&sql, params).await?;
        let result = query_result.try_get()?;
        Ok(result)
    }

    async fn select_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<Vec<T>>
    where
        T: crate::Entity + crate::TryGetable,
    {
        let (sql, params) =
            build_select_sql::<T, _>(self, |index| wrapper.build(self.platform(), index + 1))
                .await?;
        let query_result = self.pool().fetch_all(&sql, params).await?;
        let result = query_result.try_get()?;
        Ok(result)
    }

    async fn select_all<T>(&self) -> GrapefruitResult<Vec<T>>
    where
        T: crate::Entity + crate::TryGetable,
    {
        let (sql, _) = build_select_sql::<T, _>(self, |_| ("".to_owned(), vec![])).await?;
        let query_result = self.pool().fetch_all(&sql, crate::Params::Null).await?;
        let result = query_result.try_get()?;
        Ok(result)
    }

    async fn count_all<T>(&self) -> GrapefruitResult<i64>
    where
        T: crate::Entity,
    {
        let sql = format!("SELECT COUNT(1) FROM {} WHERE 1 = 1", T::table_name(),);
        let (sql, params) = build_logic_delete::<T>(sql, vec![], self.platform());
        let query_result = self.pool().fetch_one(&sql, params).await?;
        let result = query_result.try_get()?;
        Ok(result.unwrap_or(0))
    }

    async fn count_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<i64>
    where
        T: crate::Entity,
    {
        let (build_sql, vals) = wrapper.build(self.platform(), 1);
        let sql = format!(
            "SELECT COUNT(1) FROM {} WHERE {}",
            T::table_name(),
            build_sql,
        );
        let (sql, params) = build_logic_delete::<T>(sql, vals, self.platform());
        let query_result = self.pool().fetch_one(&sql, params).await?;
        let result = query_result.try_get()?;
        Ok(result.unwrap_or(0))
    }

    /// page by wrapper
    async fn page_by_wrapper<T>(
        &self,
        page: i64,
        row: i64,
        wrapper: Wrapper,
    ) -> GrapefruitResult<Page<T>>
    where
        T: crate::Entity + crate::TryGetable,
    {
        let (build_sql, vals) = wrapper.build(self.platform(), 1);
        let select_colums = T::select_columns();
        let sql = format!(
            "SELECT {} FROM {}  WHERE {}",
            select_colums.join(","),
            T::table_name(),
            build_sql,
        );
        let (sql, params) = build_logic_delete::<T>(sql, vals, self.platform());
        // 构建条数查询
        let count_sql = format!("SELECT COUNT(1) FROM ( {} ) t", sql);
        let mut page = Page::new(page, row);
        let count_query_result = self.pool().fetch_one(&count_sql, params.clone()).await?;
        let count_result: i64 = count_query_result.try_get()?.unwrap_or(0);
        page.total = count_result;
        if count_result <= 0 {
            return Ok(page);
        }

        let query_sql = format!("{} limit {} offset {} ", sql, page.limit(), page.offset());
        let query_result = self.pool().fetch_all(&query_sql, params).await?;
        page.records = query_result.try_get::<T>()?;
        Ok(page)
    }
}
