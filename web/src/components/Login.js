import React, { useState } from "react"
import { Container, Row, Col, Form, Nav, FormGroup} from 'react-bootstrap';
import styled from "styled-components";
import {PrimaryButton, OutlinedPrimaryButton} from './Buttons';
import axios from 'axios';

const Main = styled(Row)`
    text-align:left;
    padding: 6rem 0;
`;

const Title = styled.h1`
    font-weight: bold;
    font-size:3.5rem;
`;

const SubTitle = styled.h2`
    font-weight: normal;
    font-size:2rem;
`;

const Login = () => {

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [remember, setRemember] = useState("");
  
    const handleSubmit = (evt) => {
        evt.preventDefault();
        // alert(`Submitting Email ${email} and Password ${password}`)
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
            .then(data => console.log(data));
            console.log("After Post")
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
                    </Row>
                    </Col>
            </Main>
        </Container>
    );

    };
export default Login;