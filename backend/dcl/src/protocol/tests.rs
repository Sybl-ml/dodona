use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use mockito::mock;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::protocol;
use messages::{ClientMessage, WriteLengthPrefix};

#[tokio::test]
async fn nodes_can_immediately_send_tokens() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let authenticate = mock(
        "POST",
        "/api/clients/m/5fe8b9d85511355cdab720aa/authenticate",
    )
    .with_status(200)
    .with_body(r#"{"message": "Authentication successful"}"#)
    .create();

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        let token = handler.get_access_token().await.unwrap().unwrap().1;

        assert_eq!(token, "abc");
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;
    let message = ClientMessage::AccessToken {
        id: String::from("5fe8b9d85511355cdab720aa"),
        token: String::from("abc"),
    };

    // Write our access token and shutdown the stream
    stream.write(&message.as_bytes()).await?;
    stream.shutdown().await?;

    // Ensure the listener handled it correctly
    assert!(handler.await.is_ok());

    // Ensure the mock handler got called
    authenticate.assert();

    Ok(())
}

#[tokio::test]
async fn invalid_protocol_states_cause_panics() -> Result<(), Box<dyn Error>> {
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
    let message = ClientMessage::Alive { timestamp: 0 };

    // Send an invalid message as the first one
    stream.write(&message.as_bytes()).await?;
    stream.shutdown().await?;

    // Check that the handler failed to handle it
    assert!(handler.await.is_err());

    Ok(())
}

#[tokio::test]
async fn incorrect_ordering_fails() -> Result<(), Box<dyn Error>> {
    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        handler.get_access_token().await.unwrap().unwrap().1;
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;

    // Prepare messages for sending
    let challenge_response = ClientMessage::ChallengeResponse {
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

#[tokio::test]
async fn protocol_cares_about_api_responses() -> Result<(), Box<dyn Error>> {
    // Setup the API server mocking
    let authenticate = mock(
        "POST",
        "/api/clients/m/5fe8b9d85511355cdab720aa/authenticate",
    )
    .with_status(401)
    .create();

    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    let handler = tokio::spawn(async move {
        // Accept a single stream
        let mut stream = listener.accept().await.unwrap().0;

        // Setup the handler and get the access token
        let mut handler = protocol::Handler::new(&mut stream);
        let token = handler.get_access_token().await.unwrap().unwrap().1;

        assert_eq!(token, "abc");
    });

    // Wait for the handler to be ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the handler
    let mut stream = TcpStream::connect(addr).await?;
    let message = ClientMessage::AccessToken {
        id: String::from("5fe8b9d85511355cdab720aa"),
        token: String::from("abc"),
    };

    // Write our access token and shutdown the stream
    stream.write(&message.as_bytes()).await?;
    stream.shutdown().await?;

    // Ensure the listener handled it correctly
    assert!(handler.await.is_err());

    // Ensure the mock handler got called
    authenticate.assert();

    Ok(())
}
