import React, { useState } from "react"
import { Container, Row, Col, Form } from 'react-bootstrap';
import styled from "styled-components";
import {PrimaryButton} from './Buttons';
import cookies from './../Auth'; 
import { Redirect } from 'react-router-dom';

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
    const [regState, setRegState] = useState(0);
  
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
                .then(data => {
                    console.log(data.token);
                    if (data.token === "null") {
                        setRegState(2);
                    }
                    else {
                        setRegState(1);
                        cookies.set("token", data.token, { path: '/' , sameSite: true})
                    }
                });
        }
        else {
            setRegState(3);
        }

    }

    const checkRegState = () => {
        if (regState === 1) {
            console.log("Authenticated");
            return <Redirect to="/dashboard"/>;
        }
        else if (regState === 2){
            return <p>Something is wrong with the information you have provided</p>;
        }
        else if (regState === 3){
            return <p>Passwords don't match</p>;
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
                    {checkRegState()}
                    </Form>
                    </Row>
                    </Col>
            </Main>
        </Container>
    );
    };
export default Register;