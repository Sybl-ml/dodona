//! Part of DCL that deals with financial aspects of running models
use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};

use models::users::User;

/// Pricing struct to contain information about
/// the pricing of a job
#[derive(Debug, Clone, Copy)]
pub struct Pricing {
    /// Revenue for a job
    pub revenue: f64,
    /// Rate of commision charged by Sybl
    pub commision_rate: f64,
}

impl Pricing {
    /// Creates a new pricing struct
    pub fn new(revenue: f64, commision_rate: f64) -> Pricing {
        Pricing {
            revenue,
            commision_rate,
        }
    }

    /// Function to pay a client for the use of their model
    /// to compute predictions. This is based on their
    /// impact in the final result.
    pub async fn reimburse(
        &self,
        database: Arc<Database>,
        user_id: ObjectId,
        weight: f64,
    ) -> Result<()> {
        let amount: i32 = (((self.revenue - self.commision_rate) * weight) * 100.0) as i32;
        let users = database.collection("users");

        let filter = doc! { "_id": &user_id };
        let user_doc = users.find_one(filter.clone(), None).await?.unwrap();

        let mut user: User = mongodb::bson::de::from_document(user_doc)?;

        user.credits += amount;

        let document = mongodb::bson::ser::to_document(&user)?;
        users.update_one(filter, document, None).await?;

        Ok(())
    }
}
