use models::jobs::JobConfiguration;

use super::*;

fn create_job_element() -> (ObjectId, DatasetPair, Job) {
    (
        ObjectId::new(),
        DatasetPair::default(),
        Job::new(JobConfiguration::default()),
    )
}

#[test]
fn jobs_can_be_added() {
    let queue = JobQueue::new();

    queue.push(create_job_element());

    assert_eq!(queue.0.lock().unwrap().len(), 1);
}

#[test]
fn filter_finds_correct_job_indices() {
    let queue = JobQueue::new();
    let mut element = create_job_element();

    // Add an element with cluster_size = 2
    element.2.config.cluster_size = 2;
    queue.push(element.clone());

    // Add an element with cluster_size = 3
    element.2.config.cluster_size = 3;
    queue.push(element.clone());

    // Add an element with cluster_size = 1
    element.2.config.cluster_size = 1;
    queue.push(element.clone());

    // Set the number of active nodes to 2
    let active = AtomicUsize::new(2);

    assert_eq!(queue.filter(&active), vec![0, 2]);
}

#[test]
fn sometimes_no_jobs_can_be_completed() {
    let queue = JobQueue::new();
    let mut element = create_job_element();

    // Add an element with cluster_size = 4
    element.2.config.cluster_size = 4;
    queue.push(element.clone());

    // Add an element with cluster_size = 3
    element.2.config.cluster_size = 3;
    queue.push(element.clone());

    // Add an element with cluster_size = 2
    element.2.config.cluster_size = 2;
    queue.push(element.clone());

    // Set the number of active nodes to 1
    let active = AtomicUsize::new(1);

    assert_eq!(queue.filter(&active), Vec::<usize>::new());
}

#[test]
fn correct_indices_are_removed() {
    let queue = JobQueue::new();
    let mut element = create_job_element();

    // Add an element with cluster_size = 4
    element.2.config.cluster_size = 4;
    queue.push(element.clone());

    // Add an element with cluster_size = 3
    element.2.config.cluster_size = 3;
    queue.push(element.clone());

    // Remove the second job
    let removed = queue.remove(1);

    assert_eq!(removed, element);
}

#[test]
fn jobs_can_be_inserted_at_specific_indices() {
    let queue = JobQueue::new();
    let mut element = create_job_element();

    // Add an element with cluster_size = 4
    element.2.config.cluster_size = 4;
    queue.push(element.clone());

    // Add an element with cluster_size = 3
    element.2.config.cluster_size = 3;
    queue.push(element.clone());

    // Insert an element inbetween them with cluster_size = 5
    element.2.config.cluster_size = 5;
    queue.insert(1, element.clone());

    // Check the second element
    assert_eq!(queue.0.lock().unwrap()[1], element);
}
