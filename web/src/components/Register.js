import React from "react"
import { Container, Row, Col, Form } from 'react-bootstrap';
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




const Register = () => {

    const handleSubmit = (event) => {
        console.log(event.target.elements.Email.value);
        console.log(event.target.elements.Password.value);
        console.log(event.target.elements.ConfirmPassword.value);
    };

    return (
            
        <Container> 
            <Main>
            <Col>
                <Row>
                <Title>Register</Title>
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
                    <Form.Group controlId="ConfirmPassword">
                        <Form.Label>Confirm Password</Form.Label>
                        <Form.Control type="password" placeholder="Password" />
                    </Form.Group>
                    <PrimaryButton variant="primary" type="submit">
                        Register
                    </PrimaryButton>
                    </Form>
                    </Row>
                    </Col>
            </Main>
        </Container>
    );
    };
export default Register;