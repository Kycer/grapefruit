use crate::{GrapefruitResult, MetaObject, Page, Wrapper};

#[async_trait::async_trait]
pub trait IdentifierGenerator: Send + Sync {
    async fn next_id(&self) -> crate::Value;
}

pub trait MetaObjectHandler: Send + Sync {
    fn insert_fill(&self, meta: &mut MetaObject);

    fn update_fill(&self, meta: &mut MetaObject);
}

#[async_trait::async_trait]
pub trait GrapefruitRepository: Send + Sync {
    /// Insert a record
    async fn insert<T>(&self, entity: &T) -> GrapefruitResult<u64>
    where
        T: crate::Entity;

    /// Insert Batch record
    async fn insert_batch<T>(&self, entities: &[&T]) -> GrapefruitResult<u64>
    where
        T: crate::Entity;

    /// Update record by id
    async fn update_by_id<T>(&self, entity: &T) -> GrapefruitResult<u64>
    where
        T: crate::Entity;

    /// Update record by Wrapper
    async fn update_by_wrapper<T>(&self, entity: &T, wrapper: Wrapper) -> GrapefruitResult<u64>
    where
        T: crate::Entity;

    /// Delete by id
    async fn delete_by_id<T, I>(&self, id: I) -> GrapefruitResult<bool>
    where
        T: crate::Entity,
        I: crate::PrimaryKey;

    /// Delete by ids
    async fn delete_by_ids<T, I>(&self, ids: &[I]) -> GrapefruitResult<bool>
    where
        T: crate::Entity,
        I: crate::PrimaryKey;

    /// Delete by Wrapper
    async fn delete_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<bool>
    where
        T: crate::Entity;

    /// Find record by id
    async fn select_by_id<T, I>(&self, id: I) -> GrapefruitResult<Option<T>>
    where
        T: crate::Entity + crate::TryGetable,
        I: crate::PrimaryKey;

    /// Find record by wrapper
    async fn select_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<Vec<T>>
    where
        T: crate::Entity + crate::TryGetable;

    /// Find all
    async fn select_all<T>(&self) -> GrapefruitResult<Vec<T>>
    where
        T: crate::Entity + crate::TryGetable;

    /// count all
    async fn count_all<T>(&self) -> GrapefruitResult<i64>
    where
        T: crate::Entity;

    /// count by wrapper
    async fn count_by_wrapper<T>(&self, wrapper: Wrapper) -> GrapefruitResult<i64>
    where
        T: crate::Entity;

    /// page by wrapper
    async fn page_by_wrapper<T>(
        &self,
        page: i64,
        row: i64,
        wrapper: Wrapper,
    ) -> GrapefruitResult<Page<T>>
    where
        T: crate::Entity + crate::TryGetable;
}

#[async_trait::async_trait]
pub trait BaseRepository<I, T>: Send + Sync
where
    I: crate::PrimaryKey,
    T: crate::Entity + crate::TryGetable,
{
    /// get grapefruit
    async fn get_grapefruit(&self) -> &crate::Grapefruit {
        let grapefruit = crate::GRAPEFRUIT.get().expect("Grapefruit not init");
        grapefruit
    }

    /// Insert a record
    async fn insert(&self, entity: &T) -> GrapefruitResult<u64> {
        self.get_grapefruit().await.insert(entity).await
    }

    /// Insert Batch record
    async fn insert_batch(&self, entities: &[&T]) -> GrapefruitResult<u64> {
        self.get_grapefruit().await.insert_batch(entities).await
    }

    /// Update record by id
    async fn update_by_id(&self, entity: &T) -> GrapefruitResult<u64> {
        self.get_grapefruit().await.update_by_id(entity).await
    }

    /// Update record by Wrapper
    async fn update_by_wrapper(&self, entity: &T, wrapper: Wrapper) -> GrapefruitResult<u64> {
        self.get_grapefruit()
            .await
            .update_by_wrapper(entity, wrapper)
            .await
    }

    /// Delete by id
    async fn delete_by_id(&self, id: I) -> GrapefruitResult<bool> {
        self.get_grapefruit().await.delete_by_id::<T, I>(id).await
    }

    /// Delete by ids
    async fn delete_by_ids(&self, ids: &[I]) -> GrapefruitResult<bool> {
        self.get_grapefruit().await.delete_by_ids::<T, I>(ids).await
    }

    /// Delete by Wrapper
    async fn delete_by_wrapper(&self, wrapper: Wrapper) -> GrapefruitResult<bool> {
        self.get_grapefruit()
            .await
            .delete_by_wrapper::<T>(wrapper)
            .await
    }

    /// Find record by id
    async fn select_by_id(&self, id: I) -> GrapefruitResult<Option<T>> {
        self.get_grapefruit().await.select_by_id::<T, I>(id).await
    }

    /// Find record by wrapper
    async fn select_by_wrapper(&self, wrapper: Wrapper) -> GrapefruitResult<Vec<T>> {
        self.get_grapefruit()
            .await
            .select_by_wrapper::<T>(wrapper)
            .await
    }

    /// Find all
    async fn select_all(&self) -> GrapefruitResult<Vec<T>> {
        self.get_grapefruit().await.select_all::<T>().await
    }

    /// count all
    async fn count_all(&self) -> GrapefruitResult<i64> {
        self.get_grapefruit().await.count_all::<T>().await
    }

    /// count by wrapper
    async fn count_by_wrapper(&self, wrapper: Wrapper) -> GrapefruitResult<i64> {
        self.get_grapefruit()
            .await
            .count_by_wrapper::<T>(wrapper)
            .await
    }

    /// page by wrapper
    async fn page_by_wrapper(
        &self,
        page: i64,
        row: i64,
        wrapper: Wrapper,
    ) -> GrapefruitResult<Page<T>> {
        self.get_grapefruit()
            .await
            .page_by_wrapper::<T>(page, row, wrapper)
            .await
    }
}
