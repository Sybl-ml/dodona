import React from "react"
import { Container, Row, Col, Form, Nav} from 'react-bootstrap';
import styled from "styled-components";
import {PrimaryButton, OutlinedPrimaryButton} from './Buttons';

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

    const handleSubmit = (event) => {
        // Email and password should be collected. Password 
        // should be hashed and sent to API and confirm if 
        // correct. If it is correct, it will send back 
        // client token
        console.log(event.target.elements.Email.value);
        console.log(event.target.elements.Password.value);
        console.log(event.target.elements.RememberMe.value);
    };

        return (
            
            <Container fluid="xl"> 
                <Main>
                <Col xs="auto">
                    <Row>
                    <Title>Login</Title>
                    </Row>
                    <Row>
                    <Form onSubmit={event => handleSubmit(event)}>
                        <Form.Group controlId="Email">
                            <Form.Label>Email address</Form.Label>
                            <Form.Control type="email" placeholder="Enter email" />
                        </Form.Group>

                        <Form.Group controlId="Password">
                            <Form.Label>Password</Form.Label>
                            <Form.Control type="password" placeholder="Password" />
                        </Form.Group>
                        <Form.Group controlId="RememberMe">
                            <Form.Check type="checkbox" label="Remember Me" />
                        </Form.Group>
                        <Row>
                        <PrimaryButton variant="primary" type="submit">
                            Login
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