use bytes::Bytes;
use chrono::Utc;
use mongodb::bson::{self, de::from_document, doc, document::Document, oid::ObjectId};
use tokio_stream::StreamExt;

use utils::compress::decompress_data;

/// Represents the metadata of a file in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    /// The unique identifier for this file
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The length of the file, measured in bytes
    pub length: i32,
    /// The number of chunks this file is represented as
    // TODO: MongoDB uses a `chunk_size` parameter instead
    pub chunk_count: i32,
    /// The date and time the file was uploaded
    pub upload_date: bson::DateTime,
    /// The name of the file
    pub filename: String,
    /// Any user defined metadata to go with the file
    pub metadata: Document,
}

impl File {
    /// Creates a new file given the filename.
    ///
    /// Note, this only creates the instance locally, it does not store any data in the database
    /// yet.
    pub fn new(filename: String) -> Self {
        log::debug!("Creating a new file with filename={}", filename);

        Self {
            id: ObjectId::new(),
            length: 0,
            chunk_count: 0,
            upload_date: bson::DateTime(Utc::now()),
            filename,
            metadata: Document::new(),
        }
    }

    /// Uploads a new [`Chunk`] to the database and updates the internal state of the [`File`].
    pub async fn upload_chunk(
        &mut self,
        database: &mongodb::Database,
        bytes: &Bytes,
    ) -> mongodb::error::Result<()> {
        // Create a new chunk for the given data
        let chunk = Chunk::new(self.id.clone(), self.chunk_count, bytes.to_vec());

        // Update the length and chunk count
        self.length += bytes.len() as i32;
        self.chunk_count += 1;

        // Upload the chunk itself
        chunk.upload(&database).await?;

        Ok(())
    }

    /// Indicates there is no more data and uploads the [`File`] to the database.
    pub async fn finalise(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        // Get the files collection
        let files = database.collection("files");

        log::debug!(
            "Finalising a file with id={}, {} bytes of data across {} chunk(s)",
            self.id,
            self.length,
            self.chunk_count
        );

        // Convert the file to a document
        let document = bson::ser::to_document(&self).unwrap();

        // Upload it to the relevant
        files.insert_one(document, None).await?;

        Ok(())
    }

    pub async fn download_chunks(
        &self,
        database: &mongodb::Database,
    ) -> mongodb::error::Result<mongodb::Cursor<Document>> {
        let chunks = database.collection("chunks");

        let filter = doc! {"files_id": &self.id};
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! {"n": 1})
            .build();

        chunks.find(filter, options).await
    }

    pub async fn download_dataset(
        &self,
        database: &mongodb::Database,
    ) -> mongodb::error::Result<Vec<u8>> {
        let chunks = database.collection("chunks");

        let filter = doc! {"files_id": &self.id};
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! {"n": 1})
            .build();

        let mut data: Vec<u8> = Vec::new();

        let mut cursor = chunks.find(filter, options).await?;

        while let Some(next) = cursor.next().await {
            let chunk: Chunk = from_document(next?)?;
            let chunk_bytes = chunk.data.bytes;
            let decomp_data = decompress_data(&chunk_bytes).unwrap();
            data.extend_from_slice(&decomp_data);
        }

        Ok(data)
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let files = database.collection("files");
        let chunks = database.collection("chunks");

        log::debug!(
            "Deleting file with id={}, filename={}",
            self.id,
            self.filename
        );

        let filter = doc! {"files_id": &self.id};
        chunks.delete_many(filter, None).await?;

        let filter = doc! {"_id": &self.id};
        files.delete_one(filter, None).await?;

        Ok(())
    }
}

/// Represents a chunk of a file in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    /// The unique identifier for this chunk
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The identifier of the file it belongs to
    pub files_id: ObjectId,
    /// The index number of this chunk
    pub n: i32,
    /// The data stored within the chunk
    pub data: bson::Binary,
}

impl Chunk {
    /// Creates a new [`Chunk`] referring to a [`File`].
    pub fn new(files_id: ObjectId, n: i32, data: Vec<u8>) -> Self {
        log::debug!(
            "Creating chunk index={} for file_id={} with {} bytes of data",
            n,
            files_id,
            data.len()
        );

        Self {
            id: ObjectId::new(),
            files_id,
            n,
            data: bson::Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: data,
            },
        }
    }

    /// Uploads the chunk into the database.
    pub async fn upload(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let chunks = database.collection("chunks");

        // Convert the struct to a document
        let document = bson::ser::to_document(&self).unwrap();

        // Upload the document
        chunks.insert_one(document, None).await?;

        Ok(())
    }
}
