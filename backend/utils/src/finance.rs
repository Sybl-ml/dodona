//! Part of DCL that deals with financial aspects of running models
use std::cmp::max;
use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};

pub const COMMISSION_RATE: f64 = 0.25;
pub const SIZE_DENOMINATOR: i32 = 1000;

/// Reimburses a client based on their model performance.
///
/// This updates the total amount that the model has earnt in terms of credits and also updates the
/// users credit balance.
pub async fn reimburse(
    database: Arc<Database>,
    model_id: &ObjectId,
    revenue: i32,
    weight: f64,
) -> Result<()> {
    let users = database.collection("users");
    let models = database.collection("models");

    let revenue = f64::from(revenue);
    let credits = ((revenue * (1.0 - COMMISSION_RATE)) * weight) as i32;

    log::debug!(
        "Reimbursing model_id={} with {} credits, from revenue={} and weight={}",
        model_id,
        credits,
        revenue,
        weight
    );

    // Update the model with the added revenue
    let filter = doc! { "_id": &model_id };
    let update = doc! { "$inc": { "credits_earned": credits } };
    models.update_one(filter.clone(), update, None).await?;

    // Find the model itself
    let model_doc = models
        .find_one(filter, None)
        .await?
        .expect("Failed to find the model in the database");

    // Get the user identifier of the owner
    let user_id = model_doc
        .get_object_id("user_id")
        .expect("Model had no user_id field");

    log::debug!("Reimbursing user_id={} with {} credits", user_id, credits);

    // Update their account balance
    let query = doc! { "_id": &user_id };
    let update = doc! {"$inc": {"credits": credits }};

    users.update_one(query, update, None).await?;

    Ok(())
}

// Pays a `user_id` a given `amount` of credits
// The `amount` can be positive to represent payment
// or negative to represent a charge
pub async fn pay(database: Arc<Database>, user_id: &ObjectId, amount: i32) -> Result<()> {
    let users = database.collection("users");
    let query = doc! { "_id": user_id };
    let update = doc! {"$inc": {"credits": amount}};
    users.update_one(query, update, None).await?;
    Ok(())
}

pub fn job_cost(models: i32, dimensionality: i32, size: i32) -> i32 {
    (models * dimensionality * max(size / SIZE_DENOMINATOR, 1)) as i32
}
