use serde::{de::DeserializeOwned, Serialize};
use async_trait::async_trait;
use mongodb::bson::{doc, from_bson, to_bson};
use mongodb::bson::{Bson, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, Database};
use mongodb::options;
use mongodb::results::DeleteResult;
use anyhow::{Result, Error};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::Stream;
use mongodb::Cursor;

/// A cursor of model documents.
pub struct ModelCursor<T> {
    cursor: Cursor,
    marker: std::marker::PhantomData<T>,
}

impl<T: Model> ModelCursor<T> {
    pub(crate) fn new(cursor: Cursor) -> Self {
        Self{cursor, marker: std::marker::PhantomData}
    }
}

// Impl Unpin on this container as we do not care about this container staying pinned,
// only the underlying `Cursor` needs to remain pinned while we poll from this vantage point.
impl<T> Unpin for ModelCursor<T> {}

impl<T: Model> Stream for ModelCursor<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let doc = match Pin::new(&mut self.cursor).poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(Error::new(err)))),
            Poll::Ready(Some(Ok(doc))) => doc,
        };
        match Model::instance_from_document(doc) {
            Ok(model) => Poll::Ready(Some(Ok(model))),
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}



#[async_trait]
pub trait Model where Self: Serialize + DeserializeOwned{

    const COLLECTION_NAME: &'static str;

    fn id(&self) -> Option<ObjectId>;

    fn set_id(&mut self, id: ObjectId);

    fn read_concern() -> Option<options::ReadConcern> {
        None
    }

    fn write_concern() -> Option<options::WriteConcern> {
        None
    }

    fn selection_criteria() -> Option<options::SelectionCriteria> {
        None
    }

        //////////////////////////////////////////////////////////////////////////////////////////////
    // Static Layer //////////////////////////////////////////////////////////////////////////////

    /// Get a handle to this model's collection.
    ///
    /// If there are any methods available on the underlying driver's collection object which are
    /// not available on the model interface, this is how you should access them. Typically,
    /// only methods which would be modified to deal with a model instance are actually wrapped
    /// by the model interface. Everything else should be accessed via this collection method.
    ///
    /// This method uses the model's `selection_criteria`, `read_concern` & `write_concern` when
    /// constructing the collection handle.
    fn collection(db: Database) -> Collection {
        db.collection_with_options(
            Self::COLLECTION_NAME,
            options::CollectionOptions::builder()
                .selection_criteria(Self::selection_criteria())
                .read_concern(Self::read_concern())
                .write_concern(Self::write_concern())
                .build(),
        )
    }

    /// Find all instances of this model matching the given query.
    async fn find<F, O>(db: Database, filter: F, options: O) -> Result<ModelCursor<Self>>
        where
            F: Into<Option<Document>> + Send,
            O: Into<Option<options::FindOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find(filter, options)
            .await
            .map(ModelCursor::new)?)
    }

    /// Find the one model record matching your query, returning a model instance.
    async fn find_one<F, O>(db: Database, filter: F, options: O) -> Result<Option<Self>>
        where
            F: Into<Option<Document>> + Send,
            O: Into<Option<options::FindOneOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one(filter, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and deletes it, returning the original.
    async fn find_one_and_delete<O>(db: Database, filter: Document, options: O) -> Result<Option<Self>>
        where O: Into<Option<options::FindOneAndDeleteOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_delete(filter, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and replaces it, returning either the original or replaced document.
    async fn find_one_and_replace<O>(db: Database, filter: Document, replacement: Document, options: O) -> Result<Option<Self>>
        where O: Into<Option<options::FindOneAndReplaceOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_replace(filter, replacement, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    /// Finds a single document and updates it, returning either the original or updated document.
    async fn find_one_and_update<U, O>(db: Database, filter: Document, update: U, options: O) -> Result<Option<Self>>
        where
            U: Into<options::UpdateModifications> + Send,
            O: Into<Option<options::FindOneAndUpdateOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_update(filter, update, options)
            .await?
            .map(Self::instance_from_document)
            .transpose()?)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Instance Layer ////////////////////////////////////////////////////////////////////////////

    /// Save the current model instance.
    ///
    /// In order to make this method as flexible as possible, its behavior varies a little based
    /// on the input and the state of the instance.
    ///
    /// When the instance already has an ID, this method will operate purely based on the instance
    /// ID. If no ID is present, and no `filter` has been specified, then an ID will be generated.
    ///
    /// If a `filter` is specified, and no ID exists for the instance, then the filter will be used
    /// and the first document matching the filter will be replaced by this instance. This is
    /// useful when the model has unique indexes on fields which need to be the target of the save
    /// operation.
    ///
    /// **NOTE WELL:** in order to ensure needed behavior of this method, it will force `journaled`
    /// write concern.
    async fn save(&mut self, db: Database, filter: Option<Document>) -> Result<()> {
        let coll = Self::collection(db);
        let instance_doc = Self::document_from_instance(&self)?;

        // Ensure that journaling is set to true for this call, as we need to be able to get an ID back.
        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        // Handle case where instance already has an ID.
        let mut id_needs_update = false;
        let filter = match (self.id(), filter) {
            (Some(id), _) => doc!{"_id": id},
            (None, None) => {
                let new_id = ObjectId::new();
                self.set_id(new_id.clone());
                doc!{"_id": new_id}
            }
            (None, Some(filter)) => {
                id_needs_update = true;
                filter
            }
        };

        // Save the record by replacing it entirely, or upserting if it doesn't already exist.
        let opts = options::FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .write_concern(Some(write_concern))
            .return_document(Some(options::ReturnDocument::After))
            .build();
        let updated_doc = coll.find_one_and_replace(filter, instance_doc, Some(opts))
            .await?
            .ok_or_else(|| Error::msg("Server Failed To Return Updated Doc"))?;

        // Update instance ID if needed.
        if id_needs_update {
            let response_id = updated_doc.get_object_id("_id").map_err(|_| Error::msg("Server Failed To Return ObjectId"))?;
            self.set_id(response_id.clone());
        };
        Ok(())
    }

    /// Update the current model instance.
    ///
    /// This operation will always target the model instance by the instance's ID. If its ID is
    /// `None`, this method will return an error. If a filter document is provided, this method
    /// will ensure that the key `_id` is set to this model's ID.
    ///
    /// This method will consume `self`, and will return a new instance of `Self` based on the given
    /// return options (`ReturnDocument::Before | ReturnDocument:: After`).
    ///
    /// In order to provide consistent behavior, this method will also ensure that the operation's write
    /// concern `journaling` is set to `true`, so that we can receive a complete output document.
    ///
    /// If this model instance was never written to the database, this operation will return an error.
    async fn update(self, db: Database, filter: Option<Document>, update: Document, opts: Option<options::FindOneAndUpdateOptions>) -> Result<Self> {
        // Extract model's ID & use as filter for this operation.
        let id = self.id().ok_or_else(|| Error::msg("ModelId Required For Operation"))?;

        // Ensure we have a valid filter.
        let filter = match filter {
            Some(mut doc) => {
                doc.insert("_id", id);
                doc
            }
            None => doc!{"_id": id},
        };

        // Ensure that journaling is set to true for this call for full output document.
        let options = match opts {
            Some(mut options) => {
                options.write_concern = match options.write_concern {
                    Some(mut wc) => {
                        wc.journal = Some(true);
                        Some(wc)
                    },
                    None => {
                        let mut wc = Self::write_concern().unwrap_or_default();
                        wc.journal = Some(true);
                        Some(wc)
                    }
                };
                options
            },
            None => {
                let mut options = options::FindOneAndUpdateOptions::default();
                let mut wc = Self::write_concern().unwrap_or_default();
                wc.journal = Some(true);
                options.write_concern = Some(wc);
                options
            }
        };

        // Perform a FindOneAndUpdate operation on this model's document by ID.
        Ok(Self::collection(db)
            .find_one_and_update(filter, update, Some(options))
            .await?
            .map(Self::instance_from_document)
            .transpose()?
            .ok_or_else(|| Error::msg("Server Failed To Return Updated Doc"))?)
    }

    /// Delete this model instance by ID.
    ///
    /// Wraps the driver's `Collection.delete_one` method.
    async fn delete(&self, db: Database) -> Result<DeleteResult> {
        // Return an error if the instance was never saved.
        let id = self.id().ok_or_else(|| Error::msg("ModelId Required For Operation"))?;
        Ok(Self::collection(db).delete_one(doc!{"_id": id}, None).await?)
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
    // Convenience Methods ///////////////////////////////////////////////////////////////////////

    /// Attempt to serialize the given bson document into an instance of this model.
    fn instance_from_document(document: Document) -> Result<Self> {
        Ok(from_bson::<Self>(Bson::Document(document))?)
    }

    /// Attempt to serialize an instance of this model into a bson document.
    fn document_from_instance(&self) -> Result<Document> {
        match to_bson(&self)? {
            Bson::Document(doc) => Ok(doc),
            bsn => Err(Error::msg(format!("BSON ERROR: {:?}", bsn.element_type()))),
        }
    }
}