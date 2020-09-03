import React, { useState } from "react"
import { Container, Row, Col, Form } from 'react-bootstrap';
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



const Register = () => {

    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
  
    const handleSubmit = (evt) => {
        evt.preventDefault();
        if (password === confirmPassword) {
            fetch('/api/users/new', {
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
        else {
            alert("Password Don't Match")
        }

    }

    return (
            
        <Container fluid="xl"> 
            <Main>
            <Col xs="auto">
                <Row>
                <Title>Register</Title>
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
                    <Form.Group controlId="Password">
                        <Form.Label>Confirm Password</Form.Label>
                        <Form.Control type="password" placeholder="Confirm Password" onChange={e => setConfirmPassword(e.target.value)}/>
                    </Form.Group>
                    <Row>
                    <PrimaryButton variant="primary" type="submit">
                        SIGN UP
                    </PrimaryButton>
                    </Row>
                    
                    </Form>
                    </Row>
                    </Col>
            </Main>
        </Container>
    );
    };
export default Register;