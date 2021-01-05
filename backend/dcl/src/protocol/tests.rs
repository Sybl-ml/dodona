use std::error::Error;
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4};
use std::time::Duration;

use mockito::mock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::messages::Message;
use crate::protocol;

#[tokio::test]
async fn nodes_can_immediately_send_tokens() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let _m = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#);

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        let token = handler.get_access_token().await.unwrap().1;

        assert_eq!(token, "abc");
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;
    let message = Message::AccessToken {
        id: String::from("5fe8b9d85511355cdab720aa"),
        token: String::from("abc"),
    };

    // Write our access token and shutdown the stream
    stream.write(&message.as_bytes()).await?;
    stream.shutdown(Shutdown::Both)?;

    // Ensure the listener handled it correctly
    assert!(handler.await.is_ok());

    Ok(())
}

#[tokio::test]
async fn invalid_protocol_states_cause_panics() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let _m = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#);

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        handler.get_access_token().await.unwrap();
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;
    let message = Message::Alive { timestamp: 0 };

    // Send an invalid message as the first one
    stream.write(&message.as_bytes()).await?;
    stream.shutdown(Shutdown::Both)?;

    // Check that the handler failed to handle it
    assert!(handler.await.is_err());

    Ok(())
}

#[tokio::test]
async fn nodes_can_create_models_then_authenticate() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let _new = mock("POST", "/api/clients/m/new")
        .with_status(200)
        .with_body(r#"{"challenge": "empty"}"#);

    let _verify = mock("POST", "/api/clients/m/verify")
        .with_status(200)
        .with_body(r#"{"AccessToken": {"id": "", "token": "", "expires": ""}"#);

    let _authenticate = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#);

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        let token = handler.get_access_token().await.unwrap().1;

        assert_eq!(token, "abc");
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;

    // Prepare all the messages for sending
    let new_model = Message::NewModel {
        model_name: String::from("model_name"),
        email: String::from("email"),
    };

    let challenge_response = Message::ChallengeResponse {
        response: String::from("irrelevant"),
        model_name: String::from("model_name"),
        email: String::from("email"),
    };

    let access_token = Message::AccessToken {
        id: String::from("5fe8b9d85511355cdab720aa"),
        token: String::from("abc"),
    };

    // Create a buffer for reading into (and ignoring)
    let mut buffer = [0_u8; 1024];

    // Create a new model and ignore a message
    stream.write(&new_model.as_bytes()).await?;
    stream.read(&mut buffer).await?;

    // Validate the challenge and ignore a message
    stream.write(&challenge_response.as_bytes()).await?;
    stream.read(&mut buffer).await?;

    // Authenticate with the access token
    stream.write(&access_token.as_bytes()).await?;

    // Ensure the listener handled it correctly
    assert!(handler.await.is_ok());

    Ok(())
}

#[tokio::test]
async fn incorrect_ordering_fails() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let _new = mock("POST", "/api/clients/m/new")
        .with_status(200)
        .with_body(r#"{"challenge": "empty"}"#);

    let _verify = mock("POST", "/api/clients/m/verify")
        .with_status(200)
        .with_body(r#"{"AccessToken": {"id": "", "token": "", "expires": ""}"#);

    let _authenticate = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#);

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        handler.get_access_token().await.unwrap().1;
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;

    // Prepare messages for sending
    let challenge_response = Message::ChallengeResponse {
        response: String::from("irrelevant"),
        model_name: String::from("model_name"),
        email: String::from("email"),
    };

    // Validate the challenge first, incorrect ordering here
    stream.write(&challenge_response.as_bytes()).await?;

    // Ensure the listener failed to handle it
    assert!(handler.await.is_err());

    Ok(())
}
