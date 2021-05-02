//! Contains the queue used to store upcoming jobs.

use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use mongodb::bson::oid::ObjectId;

use models::jobs::Job;

use crate::DatasetPair;

/// Struct to hold Job Queue for DCL
#[derive(Debug, Default, Clone)]
pub struct JobQueue(Arc<Mutex<VecDeque<(ObjectId, DatasetPair, Job)>>>);

impl JobQueue {
    /// Creates a new instance of the [`JobQueue`] struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Goes through jobs in the [`JobQueue`] and returns a vector
    /// containing the indexes of the jobs which the DCL can execute with
    /// its current `active` nodes.
    pub fn filter(&self, active: &AtomicUsize) -> Vec<usize> {
        let jq_mutex = self.0.lock().unwrap();
        let nodes = active.load(Ordering::SeqCst);

        let indices: Vec<_> = jq_mutex
            .iter()
            .enumerate()
            .filter(|(_, (_, _, job))| (job.config.cluster_size as usize) <= nodes)
            .map(|(idx, _)| idx)
            .collect();

        log::debug!(
            "Job queue contains {} elements, of which {} are completable with {} nodes",
            jq_mutex.len(),
            indices.len(),
            nodes
        );

        indices
    }

    /// Using an index, this function will remove the required job from the [`JobQueue`]. This is so that
    /// it gives an ownership of the data to the caller of the function.
    pub fn remove(&self, index: usize) -> (ObjectId, DatasetPair, Job) {
        let mut jq_mutex = self.0.lock().unwrap();

        jq_mutex
            .remove(index)
            .expect("Tried to get value from invalid index")
    }

    /// Puts a job back in the [`JobQueue`] if it is not being executed. This will place it in a location
    /// specified by the index parameter. This will be the place in the [`JobQueue`] that it
    /// previously was.
    pub fn insert(&self, index: usize, job: (ObjectId, DatasetPair, Job)) {
        let mut jq_mutex = self.0.lock().unwrap();

        jq_mutex.insert(index, job);
    }

    /// Enables a job to be pushed onto the end of the [`JobQueue`] when it
    /// arrives in the DCL.
    pub fn push(&self, job: (ObjectId, DatasetPair, Job)) {
        let mut job_queue_write = self.0.lock().unwrap();

        job_queue_write.push_back(job);
    }
}

#[cfg(test)]
mod tests;
