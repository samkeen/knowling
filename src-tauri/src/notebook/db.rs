use arrow_array::types::Float32Type;
use arrow_array::{ArrayRef, FixedSizeListArray, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{ArrowError, DataType, Field, Schema};
use fastembed::{Embedding, TextEmbedding};
use futures::TryStreamExt;
use lancedb::connection::Connection;
use lancedb::index::MetricType;
use lancedb::{connect, Table};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

const DB_DIR: &str = "data";
const DB_NAME: &str = "sample-lancedb";
const TABLE_NAME: &str = "documents";
const EMBEDDING_DIMENSIONS: usize = 384;
const COLUMN_ID: &str = "id";
const COLUMN_EMBEDDINGS: &str = "embeddings";
const COLUMN_TEXT: &str = "text";

pub struct Document {
    pub id: String,
    pub text: String,
}
pub struct EmbedStore {
    embedding_model: TextEmbedding,
    db_conn: Connection,
    table: Table,
}

#[derive(Debug)]
pub enum EmbedStoreError {
    VectorDb(lancedb::error::Error),
    Arrow(ArrowError),
    Embedding(anyhow::Error),
}
impl fmt::Display for EmbedStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmbedStoreError::VectorDb(e) => write!(f, "{}", e),
            EmbedStoreError::Arrow(e) => write!(f, "{}", e),
            EmbedStoreError::Embedding(e) => write!(f, "{}", e),
        }
    }
}
impl Error for EmbedStoreError {
    // Implement this to return the lower level source of this Error.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EmbedStoreError::VectorDb(e) => Some(e),
            EmbedStoreError::Arrow(e) => Some(e),
            EmbedStoreError::Embedding(_) => None,
        }
    }
}
impl From<lancedb::error::Error> for EmbedStoreError {
    fn from(e: lancedb::error::Error) -> Self {
        EmbedStoreError::VectorDb(e)
    }
}

impl From<ArrowError> for EmbedStoreError {
    fn from(e: ArrowError) -> Self {
        EmbedStoreError::Arrow(e)
    }
}

impl From<anyhow::Error> for EmbedStoreError {
    fn from(e: anyhow::Error) -> Self {
        EmbedStoreError::Embedding(e)
    }
}

impl EmbedStore {
    pub async fn new(embedding_model: TextEmbedding) -> Result<EmbedStore, EmbedStoreError> {
        let db_conn = Self::init_db_conn().await?;
        let table = Self::get_or_create_table(&db_conn, TABLE_NAME).await?;
        Ok(EmbedStore {
            embedding_model,
            db_conn,
            table,
        })
    }

    pub async fn add(
        &self,
        text: Vec<String>,
        alt_ids: Vec<String>,
    ) -> Result<(), EmbedStoreError> {
        let embeddings = self.create_embeddings(&text)?;
        assert_eq!(
            embeddings[0].len(),
            EMBEDDING_DIMENSIONS,
            "Embedding dimensions mismatch"
        );
        let schema = self.table.schema().await?;
        let records_iter = self
            .create_record_batch(embeddings, text, alt_ids, schema.clone())
            .into_iter()
            .map(Ok);

        let batches = RecordBatchIterator::new(records_iter, schema.clone());
        self.table
            .add(Box::new(batches))
            .execute()
            .await
            .map_err(EmbedStoreError::from)
    }

    pub async fn record_count(&self) -> Result<usize, EmbedStoreError> {
        self.table
            .count_rows(None)
            .await
            .map_err(EmbedStoreError::from)
    }

    pub async fn search(
        &self,
        search_text: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Document>, EmbedStoreError> {
        let limit = limit.unwrap_or(25);
        let query = self.create_embeddings(&[search_text.to_string()])?;
        // flattening a 2D vector into a 1D vector. This is necessary because the search
        // function of the Table trait expects a 1D vector as input. However, the
        // create_embeddings function returns a 2D vector (a vector of embeddings,
        // where each embedding is itself a vector)
        let query: Vec<f32> = query
            .into_iter()
            .flat_map(|embedding| embedding.to_vec())
            .collect();
        self.execute_query(Some(query), None, Some(limit)).await
    }

    async fn execute_query(
        &self,
        query: Option<Vec<f32>>,
        filter: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<Document>, EmbedStoreError> {
        let limit = limit.unwrap_or(10);
        let mut query_builder = self.table.query().limit(limit);
        if let Some(query_values) = query {
            query_builder = query_builder
                .nearest_to(&query_values)
                .metric_type(MetricType::L2);
        }
        if let Some(filter_clause) = filter {
            query_builder = query_builder.filter(&filter_clause);
        }
        let stream = query_builder.execute_stream().await?;
        // @TODO having trouble converting this error
        // I get `the trait std::convert::From<lance_core::error::Error> is not implemented for db::EmbedStoreError`
        // BUT if I add it, I get an error stating EmbedStoreError has `lance_core::Error` implemented but not `lance_core::error::Error`
        // Oddly, `Error` is directly under core_lance (in `error.rs`)
        match stream.try_collect::<Vec<_>>().await {
            Ok(result) => Ok(self.record_to_document(result)),
            Err(e) => Err(EmbedStoreError::VectorDb(lancedb::error::Error::Runtime {
                message: e.to_string(),
            })),
        }
    }
    pub async fn get(&self, id: &str) -> Result<Option<Document>, EmbedStoreError> {
        let filter = format!("id = '{}'", id);
        let mut result = self.execute_query(None, Some(filter), None).await?;
        assert!(
            result.len() <= 1,
            "The get by id method should only return one item at most"
        );
        Ok(result.pop())
    }

    pub async fn get_all(&self) -> Result<Vec<Document>, EmbedStoreError> {
        self.execute_query(None, None, Some(1000)).await
    }

    pub async fn delete<T: fmt::Display>(&self, id: T) -> Result<(), EmbedStoreError> {
        self.table
            .delete(format!("id > {id}").as_str())
            .await
            .map_err(EmbedStoreError::from)
    }

    pub async fn update(&self, id: &str, text: Vec<String>) -> Result<(), EmbedStoreError> {
        self.delete(&id).await?;
        self.add(text, vec![id.to_string()]).await
    }

    /// Creates an index on a given field.
    pub async fn create_index(&self, num_partitions: Option<u32>) -> Result<(), EmbedStoreError> {
        let num_partitions = num_partitions.unwrap_or(8);
        self.table
            .create_index(&[COLUMN_EMBEDDINGS])
            .ivf_pq()
            .num_partitions(num_partitions)
            .build()
            .await
            .map_err(EmbedStoreError::from)
    }

    fn record_to_document(&self, record_batch: Vec<RecordBatch>) -> Vec<Document> {
        let mut documents: Vec<Document> = Vec::new();
        if record_batch.len() == 0 {
            return vec![];
        }
        let first_record_batch = record_batch.first().expect("There should always be one");
        let ids = first_record_batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Failed to downcast");
        let texts = first_record_batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("Failed to downcast");
        let _x = first_record_batch.num_rows();
        (0..first_record_batch.num_rows()).for_each(|index| {
            let id = ids.value(index).to_string();
            let text = texts.value(index).to_string();
            documents.push(Document { id: id, text: text })
        });
        documents
    }

    async fn init_db_conn() -> Result<Connection, EmbedStoreError> {
        let db_path = format!("{}/{}", DB_DIR, DB_NAME);
        let db_conn = connect(db_path.as_str()).execute().await?;
        Ok(db_conn)
    }

    fn create_embeddings(&self, documents: &[String]) -> Result<Vec<Embedding>, EmbedStoreError> {
        self.embedding_model
            .embed(documents.to_vec(), None)
            .map_err(EmbedStoreError::from)
    }

    /// Transforms a 2D vector into a 2D vector where each element is wrapped in an `Option`.
    ///
    /// This function takes a 2D vector `source` as input and returns a new 2D vector where each element
    /// is wrapped in an `Option`.
    /// The outer vector is also wrapped in an `Option`. This is useful when you want to represent the
    /// absence of data in your vector.
    ///
    /// # Arguments
    ///
    /// * `source` - A 2D vector that will be transformed.
    ///
    /// # Returns
    ///
    /// A 2D vector where each element is wrapped in an `Option`, and the outer vector is also wrapped in an `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// let source = vec![vec![1, 2, 3], vec![4, 5, 6]];
    /// let result = wrap_in_option(source);
    /// assert_eq!(result, vec![Some(vec![Some(1), Some(2), Some(3)]), Some(vec![Some(4), Some(5), Some(6)])]);
    /// ```
    fn wrap_in_option<T>(&self, source: Vec<Vec<T>>) -> Vec<Option<Vec<Option<T>>>> {
        source
            .into_iter()
            .map(|inner_vec| Some(inner_vec.into_iter().map(|item| Some(item)).collect()))
            .collect()
    }

    /// Creates a record batch from a list of embeddings and a correlated list of original text.
    fn create_record_batch(
        &self,
        embeddings: Vec<Vec<f32>>,
        text: Vec<String>,
        alt_ids: Vec<String>,
        schema: Arc<Schema>,
    ) -> Vec<RecordBatch> {
        let dimensions_count = embeddings[0].len();
        let wrapped_source = self.wrap_in_option(embeddings);
        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                // id field
                // Arc::new(Int32Array::from_iter_values(0..total_records_count as i32)),
                // Embeddings field
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        wrapped_source,
                        dimensions_count as i32,
                    ),
                ),
                // Text field
                Arc::new(Arc::new(StringArray::from(text)) as ArrayRef),
                // Alt Id
                Arc::new(Arc::new(StringArray::from(alt_ids)) as ArrayRef),
            ],
        );
        match record_batch {
            Ok(batch) => {
                vec![batch]
            }
            Err(e) => {
                panic!("Was unable to create a record batch: {}", e)
            }
        }
    }

    fn generate_schema(dimensions_count: usize) -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new(
                COLUMN_EMBEDDINGS,
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dimensions_count as i32,
                ),
                true,
            ),
            Field::new(COLUMN_TEXT, DataType::Utf8, false),
            Field::new(COLUMN_ID, DataType::Utf8, false),
        ]))
    }

    async fn get_or_create_table(db_conn: &Connection, table_name: &str) -> lancedb::Result<Table> {
        let table_names = db_conn.table_names().execute().await?;
        let table = table_names.iter().find(|&name| name == table_name);
        match table {
            Some(_) => {
                let table = db_conn.open_table(table_name).execute().await?;
                Ok(table)
            }
            None => {
                let schema = Self::generate_schema(EMBEDDING_DIMENSIONS);
                let batches = RecordBatchIterator::new(vec![], schema.clone());
                let table = db_conn
                    .create_table(table_name, Box::new(batches))
                    .execute()
                    .await?;
                Ok(table)
            }
        }
    }

    /// Creates an empty table with a schema.
    async fn create_empty_table(
        &self,
        table_name: &str,
        dimensions_count: usize,
    ) -> lancedb::Result<Table> {
        let schema = Self::generate_schema(dimensions_count);
        let batches = RecordBatchIterator::new(vec![], schema.clone());
        self.db_conn
            .create_table(table_name, Box::new(batches))
            .execute()
            .await
    }
}
