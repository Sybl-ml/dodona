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

/// Function to pay a client for the use of their model
/// to compute predictions. This is based on their
/// impact in the final result.
pub async fn reimburse(
    database: Arc<Database>,
    user_id: &ObjectId,
    revenue: i32,
    weight: f64,
) -> Result<()> {
    let revenue = revenue as f64;
    let users = database.collection("users");
    let query = doc! { "_id": user_id };
    let update = doc! {"$inc": {"credits": (((revenue - (revenue * COMMISSION_RATE)) * weight) * 100.0) as i32}};
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
