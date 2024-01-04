#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Page<T>
where
    T: Sized,
{
    pub total: i64,
    pub page: i64,
    pub rows: i64,
    pub records: Vec<T>,
}

impl<T> Page<T>
where
    T: Sized,
{
    pub fn new(page: i64, rows: i64) -> Self {
        Self {
            total: 0,
            page,
            rows,
            records: vec![],
        }
    }

    pub fn limit(&self) -> i64 {
        self.rows
    }

    pub fn offset(&self) -> i64 {
        self.rows * (self.page - 1)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn map<R, F: FnMut(&T) -> R>(&self, mut f: F) -> Page<R> {
        if self.is_empty() {}
        let mut page = Page::<R>::new(self.page, self.rows);
        let result = self
            .records
            .iter()
            .map(|record| f(record))
            .collect::<Vec<_>>();
        page.records = result;
        page
    }
}
