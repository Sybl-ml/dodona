import React, { useState } from "react"
import { Container, Row, Col, Form, Nav} from 'react-bootstrap';
import { Redirect } from 'react-router-dom';
import styled from "styled-components";
import {PrimaryButton} from './Buttons';

const Main = styled(Row)`
    text-align:left;
    padding: 6rem 0;
`;

const Title = styled.h1`
    font-weight: bold;
    font-size:3.5rem;
`;

const TokenContext = React.createContext("");

const Login = () => {

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [remember, setRemember] = useState("");
    const [loginState, setLoginState] = useState(0);
  
    const handleSubmit = (evt) => {
        evt.preventDefault();
        console.log(remember);
        fetch('/api/users/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                email: email,
                password: password,
            })
            })
            .then(response => response.json())
            .then(data => {
                console.log(data.token);
                if (data.token === "null") {
                    setLoginState(2);
                }
                else {
                    setLoginState(1);
                    return <TokenContext.Provider value={data.token}/>
                }
            });
            console.log("After Post");

    }

    const checkLoginState = () => {
        if (loginState === 1) {
            console.log("Authenticated");
            return <Redirect to="/dashboard"/>;
        }
        else if (loginState === 2){
            console.log("Not Authenticated");
            return <h1>Something is wrong with your login information</h1>;
        }
    }

    return (
            
        <Container fluid="xl"> 
            <Main>
            <Col xs="auto">
                <Row>
                <Title>Login</Title>
                </Row>
                <Row>
                <Form onSubmit={handleSubmit}>
                    <Form.Group controlId="Email">
                        <Form.Label>Email address</Form.Label>
                        <Form.Control type="email" placeholder="Enter email" onChange={e => setEmail(e.target.value)}/>
                    </Form.Group>

                    <Form.Group controlId="Password">
                        <Form.Label>Password</Form.Label>
                        <Form.Control type="password" placeholder="Password" onChange={e => setPassword(e.target.value)}/>
                    </Form.Group>
                    <Form.Group controlId="RememberMe">
                        <Form.Check type="checkbox" label="Remember Me" onChange={e => setRemember(e.target.value)}/>
                    </Form.Group>
                    <Row>
                    <PrimaryButton variant="primary" type="submit">
                        LOGIN
                    </PrimaryButton>
                    <Nav>
                        <Nav.Link href="/register">Sign Up</Nav.Link>
                    </Nav>
                    </Row>
                    
                    </Form>
                    {checkLoginState()}
                    </Row>
                    </Col>
            </Main>
        </Container>
    );

    };
export default Login;