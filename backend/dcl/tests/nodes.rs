use dcl::node_end::NodePool;

#[tokio::test]
pub async fn test_choose_random_model() {
    let mut nodes: Vec<(String, f64)> = vec![
        (String::from("m1"), 0.2),
        (String::from("m2"), 0.3),
        (String::from("m3"), 0.4),
        (String::from("m4"), 0.5),
        (String::from("m5"), 0.6),
        (String::from("m7"), 0.7),
    ];
    let mut better_nodes: Vec<(String, f64)> = vec![
        (String::from("m4"), 0.51),
        (String::from("m5"), 0.6),
        (String::from("m7"), 0.7),
    ];
    let mut cluster_performance = 0.0;

    // Base Condition
    let taken = NodePool::choose_random_node(&mut nodes, &mut better_nodes, cluster_performance);
    assert_eq!(better_nodes.contains(&taken), nodes.contains(&taken));

    // Lower than 0.5
    cluster_performance = 0.4;
    let taken = NodePool::choose_random_node(&mut nodes, &mut better_nodes, cluster_performance);
    assert_eq!(better_nodes.contains(&taken), nodes.contains(&taken));
    assert!(taken.1 > 0.5);

    // Above 0.5
    cluster_performance = 0.7;
    let taken = NodePool::choose_random_node(&mut nodes, &mut better_nodes, cluster_performance);
    assert_eq!(better_nodes.contains(&taken), nodes.contains(&taken));
}
