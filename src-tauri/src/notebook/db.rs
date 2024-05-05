use std::fmt;
use std::path::Path;
use std::sync::Arc;

use arrow_array::{
    ArrayRef, FixedSizeListArray, Float32Array, Int64Array, RecordBatch, RecordBatchIterator,
    StringArray,
};
use arrow_array::types::Float32Type;
use arrow_schema::{ArrowError, DataType, Field, Schema};
use chrono::Utc;
use fastembed::{Embedding, TextEmbedding};
use futures::TryStreamExt;
use lancedb::{connect, Table};
use lancedb::connection::Connection;
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Serialize;
use thiserror::Error;

const DB_NAME: &str = "lancedb_storage";
const TABLE_NAME: &str = "documents";
const EMBEDDING_DIMENSIONS: usize = 384;
const COLUMN_ID: &str = "id";
const COLUMN_EMBEDDINGS: &str = "embeddings";
const COLUMN_TEXT: &str = "text";

pub trait Documentable {
    fn id(&self) -> &str;
    fn text(&self) -> &str;
    fn created(&self) -> i64;
    fn modified(&self) -> i64;

    fn set_id(&mut self, id: String);
    fn set_text(&mut self, text: String);
    fn set_created(&mut self, created: i64);
    fn set_modified(&mut self, modified: i64);
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Document {
    id: String,
    text: String,
    created: i64,
    modified: i64,
}

impl Document {
    pub fn new(text: &str) -> Self {
        let now = Utc::now().timestamp();
        Document {
            id: Self::generate_id(),
            text: text.to_string(),
            created: now,
            modified: now,
        }
    }

    /// Returns the id of the Note.
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the content of the Note.
    pub fn get_content(&self) -> &str {
        &self.text
    }

    /// Generates a random id for a Document.
    /// The id is a 6-character string composed of alphanumeric characters.
    fn generate_id() -> String {
        let mut rng = rand::thread_rng();
        let id: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(6)
            .collect();
        id
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Documentable for Document {
    fn id(&self) -> &str {
        &self.id
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn created(&self) -> i64 {
        self.created
    }

    fn modified(&self) -> i64 {
        self.modified
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn set_created(&mut self, created: i64) {
        self.created = created;
    }

    fn set_modified(&mut self, modified: i64) {
        self.modified = modified;
    }
}

pub struct EmbedStore {
    embedding_model: TextEmbedding,
    db_conn: Connection,
    table: Table,
}

#[derive(Error, Debug)]
pub enum EmbedStoreError {
    #[error("Vector database error: {0}")]
    VectorDb(#[from] lancedb::error::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] ArrowError),

    #[error("Embedding error: {0}")]
    Embedding(#[from] anyhow::Error),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl Serialize for EmbedStoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        match self {
            EmbedStoreError::VectorDb(err) => serializer.serialize_newtype_variant(
                "EmbedStoreError",
                0,
                "VectorDb",
                &err.to_string(),
            ),
            EmbedStoreError::Arrow(err) => serializer.serialize_newtype_variant(
                "EmbedStoreError",
                1,
                "Arrow",
                &err.to_string(),
            ),
            EmbedStoreError::Embedding(err) => serializer.serialize_newtype_variant(
                "EmbedStoreError",
                2,
                "Embedding",
                &err.to_string(),
            ),
            EmbedStoreError::Runtime(err) => {
                serializer.serialize_newtype_variant("EmbedStoreError", 3, "Runtime", err)
            }
        }
    }
}

impl EmbedStore {
    pub async fn new(
        embedding_model: TextEmbedding,
        storage_path: &Path,
    ) -> Result<EmbedStore, EmbedStoreError> {
        let db_conn = Self::init_db_conn(storage_path).await?;
        let table = Self::get_or_create_table(&db_conn, TABLE_NAME).await?;
        Ok(EmbedStore {
            embedding_model,
            db_conn,
            table,
        })
    }

    pub async fn add<D: Documentable>(&self, documents: Vec<D>) -> Result<(), EmbedStoreError> {
        let alt_ids: Vec<String> = documents.iter().map(|doc| doc.id().to_string()).collect();
        let text: Vec<String> = documents.iter().map(|doc| doc.text().to_string()).collect();
        let created: Vec<i64> = documents.iter().map(|doc| doc.created()).collect();
        let modified: Vec<i64> = documents.iter().map(|doc| doc.modified()).collect();

        log::info!("Saving Documents: {:?}", alt_ids);
        let embeddings = self.create_embeddings(&text)?;
        assert_eq!(
            embeddings[0].len(),
            EMBEDDING_DIMENSIONS,
            "Embedding dimensions mismatch"
        );

        self.add_records(alt_ids, text, embeddings, created, modified)
            .await
    }

    async fn add_records(
        &self,
        alt_ids: Vec<String>,
        text: Vec<String>,
        embeddings: Vec<Embedding>,
        created: Vec<i64>,
        modified: Vec<i64>,
    ) -> Result<(), EmbedStoreError> {
        let schema = self.table.schema().await?;
        let records_iter = self
            .create_record_batch(embeddings, text, alt_ids, created, modified, schema.clone())
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

    pub async fn search<D: Documentable + Default>(
        &self,
        search_text: &str,
        filter: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<(D, f32)>, EmbedStoreError> {
        let query = self.create_embeddings(&[search_text.to_string()])?;
        // flattening a 2D vector into a 1D vector. This is necessary because the search
        // function of the Table trait expects a 1D vector as input. However, the
        // create_embeddings function returns a 2D vector (a vector of embeddings,
        // where each embedding is itself a vector)
        let query: Vec<f32> = query
            .into_iter()
            .flat_map(|embedding| embedding.to_vec())
            .collect();
        self.execute_search(query, filter, limit).await
    }

    async fn execute_search<D: Documentable + Default>(
        &self,
        query: Vec<f32>,
        filter: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<(D, f32)>, EmbedStoreError> {
        // let limit = limit.unwrap_or(25);
        let mut query_builder = self
            .table
            .vector_search(query)
            .map_err(EmbedStoreError::VectorDb)?;
        if let Some(filter_clause) = filter {
            query_builder = query_builder.only_if(filter_clause);
        }
        if let Some(limit_clause) = limit {
            query_builder = query_builder.limit(limit_clause);
        }

        let stream = query_builder.execute().await?;
        let record_batches = match stream.try_collect::<Vec<_>>().await {
            Ok(batches) => batches,
            Err(err) => {
                return Err(EmbedStoreError::VectorDb(lancedb::error::Error::Runtime {
                    message: err.to_string(),
                }));
            }
        };
        let documents = self.record_to_document_with_distances(record_batches)?;

        Ok(documents)
    }

    async fn execute_query<D: Documentable + Default>(
        &self,
        filter: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<D>, EmbedStoreError> {
        let mut query_builder = self.table.query();
        if let Some(filter_clause) = filter {
            query_builder = query_builder.only_if(filter_clause);
        }
        if let Some(limit_clause) = limit {
            query_builder = query_builder.limit(limit_clause);
        }

        let stream = query_builder.execute().await?;

        let record_batches = match stream.try_collect::<Vec<_>>().await {
            Ok(batches) => batches,
            Err(err) => {
                return Err(EmbedStoreError::VectorDb(lancedb::error::Error::Runtime {
                    message: err.to_string(),
                }));
            }
        };

        let documents = self.record_to_document(record_batches)?;
        Ok(documents)
    }
    pub async fn get<D: Documentable + Default>(
        &self,
        id: &str,
    ) -> Result<Option<D>, EmbedStoreError> {
        let filter = format!("id = '{}'", id);
        let mut result = self.execute_query(Some(&filter), None).await?;
        assert!(
            result.len() <= 1,
            "The get by id method should only return one item at most"
        );
        Ok(result.pop())
    }

    pub async fn get_all<D: Documentable + Default>(
        &self,
    ) -> Result<(Vec<D>, usize), EmbedStoreError> {
        let total_records = self.record_count().await?;
        let documents = self.execute_query(None, None).await?;
        log::info!(
            "get_all returned {} records. Total rows in db: {}",
            documents.len(),
            total_records
        );

        Ok((documents, total_records))
    }

    pub async fn delete<T: fmt::Display>(&self, ids: &Vec<T>) -> Result<(), EmbedStoreError> {
        let comma_separated = ids
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        self.table
            .delete(format!("id in ('{}')", comma_separated).as_str())
            .await
            .map_err(EmbedStoreError::from)
    }

    pub async fn update<D: Documentable>(
        &self,
        mut documents: Vec<D>,
    ) -> Result<(), EmbedStoreError> {
        let ids: Vec<String> = documents
            .iter()
            .map(|doc| doc.id().to_string().clone())
            .collect();
        self.delete(&ids).await?;

        let now = Utc::now().timestamp();
        documents.iter_mut().for_each(|doc| doc.set_modified(now));

        self.add(documents).await
    }

    /// Creates an index on a given field.
    pub async fn create_index(&self) -> Result<(), EmbedStoreError> {
        self.table
            .create_index(&[COLUMN_EMBEDDINGS], Index::Auto)
            .execute()
            .await
            .map_err(EmbedStoreError::from)
    }

    fn record_to_document_with_distances<D: Documentable + Default>(
        &self,
        record_batches: Vec<RecordBatch>,
    ) -> Result<Vec<(D, f32)>, EmbedStoreError> {
        let mut docs_with_distance: Vec<(D, f32)> = Vec::new();
        if record_batches.is_empty() {
            return Ok(vec![]);
        }
        for record_batch in record_batches {
            let ids = self.downcast_column::<StringArray>(&record_batch, "id")?;
            let texts = self.downcast_column::<StringArray>(&record_batch, "text")?;
            let distances = self.downcast_column::<Float32Array>(&record_batch, "_distance")?;
            let created_values = self.downcast_column::<Int64Array>(&record_batch, "created")?;
            let modified_values = self.downcast_column::<Int64Array>(&record_batch, "modified")?;

            (0..record_batch.num_rows()).for_each(|index| {
                let id = ids.value(index).to_string();
                let text = texts.value(index).to_string();
                let distance = distances.value(index);
                let created = created_values.value(index);
                let modified = modified_values.value(index);
                let mut document = D::default();
                document.set_id(id);
                document.set_text(text);
                document.set_created(created);
                document.set_modified(modified);

                docs_with_distance.push((document, distance));
            });
        }
        log::info!(
            "Converted [{}] batch results to Documents",
            docs_with_distance.len()
        );

        Ok(docs_with_distance)
    }

    fn record_to_document<D: Documentable + Default>(
        &self,
        record_batches: Vec<RecordBatch>,
    ) -> Result<Vec<D>, EmbedStoreError> {
        let mut documents: Vec<D> = Vec::new();
        if record_batches.is_empty() {
            return Ok(vec![]);
        }
        for record_batch in record_batches {
            let ids = self.downcast_column::<StringArray>(&record_batch, "id")?;
            let texts = self.downcast_column::<StringArray>(&record_batch, "text")?;
            let created_values = self.downcast_column::<Int64Array>(&record_batch, "created")?;
            let modified_values = self.downcast_column::<Int64Array>(&record_batch, "modified")?;

            (0..record_batch.num_rows()).for_each(|index| {
                let id = ids.value(index).to_string();
                let text = texts.value(index).to_string();
                let created = created_values.value(index);
                let modified = modified_values.value(index);
                let mut document = D::default();
                document.set_id(id);
                document.set_text(text);
                document.set_created(created);
                document.set_modified(modified);
                documents.push(document);
            });
        }
        log::info!("Converted [{}] batch results to Documents", documents.len());
        Ok(documents)
    }

    async fn init_db_conn(data_dir: &Path) -> Result<Connection, EmbedStoreError> {
        let db_path = data_dir.join(DB_NAME);
        log::info!("Connecting to db at path: '{:?}'", db_path);
        match db_path.to_str() {
            None => Err(EmbedStoreError::Runtime(format!(
                "Failed to convert db_path: {:?} to string",
                db_path
            ))),
            Some(path) => Ok(connect(path).execute().await?),
        }
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
        created: Vec<i64>,
        modified: Vec<i64>,
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
                Arc::new(Arc::new(Int64Array::from(created)) as ArrayRef),
                Arc::new(Arc::new(Int64Array::from(modified)) as ArrayRef),
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
            Field::new("created", DataType::Int64, false),
            Field::new("modified", DataType::Int64, false),
        ]))
    }

    async fn get_or_create_table(db_conn: &Connection, table_name: &str) -> lancedb::Result<Table> {
        let table_names = db_conn.table_names().execute().await?;
        log::info!("Existing tables: {:?}", table_names);
        let table = table_names.iter().find(|&name| name == table_name);
        match table {
            Some(_) => {
                log::info!("Connecting to existing table '{}'", table_name);
                let table = db_conn.open_table(table_name).execute().await?;
                Ok(table)
            }
            None => {
                log::info!("Table '{}' not found, creating new table", table_name);
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

    fn downcast_column<'a, T: std::fmt::Debug + 'static>(
        &self,
        record_batch: &'a RecordBatch,
        column_name: &str,
    ) -> Result<&'a T, EmbedStoreError> {
        record_batch
            .column_by_name(column_name)
            .ok_or_else(|| EmbedStoreError::Runtime(format!("{} column not found", column_name)))?
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| {
                EmbedStoreError::Runtime(format!("Failed downcasting {} column", column_name))
            })
    }
}
