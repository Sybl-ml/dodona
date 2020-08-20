import React from "react"
import { Container, Row, Col, Button, Card, } from 'react-bootstrap';
import styled from "styled-components";
import { FaExternalLinkAlt } from 'react-icons/fa';
import { SiTensorflow, SiKeras } from 'react-icons/si';
import MemoPlaceholder from '../icons/Placeholder.js';

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

const Quote = styled.h5`
    font-weight: bold;
    font-size:1rem;
    padding: 2rem 0;
`;

const FixButton = styled(Button)`
    width: auto;
    margin-right:1rem;
`;

const Welcome= () => {

        return (
            <>
            
            <Container> 
                <Main>
                    <Col>
                        <Row>
                            <Title>Empower your data with intuative Machine Learning</Title>
                        </Row>
                        <Row>
                            <SubTitle>
                                Run complex models without any infrastructure or programming experience. 
                                
                                Works with <SiTensorflow /> & <SiKeras />
                            </SubTitle>
                        </Row>
                        <Row>
                            <FixButton variant="primary">GET STARTED</FixButton>
                            <FixButton variant="primary" className="outline">
                                <FaExternalLinkAlt /> EXAMPLE
                            </FixButton>      
                        </Row>
                    </Col>
                    <Col>
                        <MemoPlaceholder />
                    </Col>
                </Main>
            </Container>

            <div className="highlight">
                <Quote className="highlighted-text">TRUSTED BY MANY ACROSS THE GLOBE</Quote>
                <Row className="justify-content-md-center">
                    <Col md="auto">
                        <Card style={{ width: '18rem' }} >
                            <Card.Body>
                                <Card.Title>Card Title</Card.Title>
                                <Card.Text>
                                Some quick example text to build on the card title and make up the bulk of
                                the card's content.
                                </Card.Text>
                                <Button variant="primary">Go somewhere</Button>
                            </Card.Body>
                        </Card>
                    </Col>
                    <Col md="auto">
                        <Card style={{ width: '18rem' }}>
                            <Card.Body>
                                <Card.Title>Card Title</Card.Title>
                                <Card.Text>
                                Some quick example text to build on the card title and make up the bulk of
                                the card's content.
                                </Card.Text>
                                <Button variant="primary">Go somewhere</Button>
                            </Card.Body>
                        </Card>
                    </Col>
                    <Col md="auto">
                        <Card style={{ width: '18rem' }}>
                            <Card.Body>
                                <Card.Title>Card Title</Card.Title>
                                <Card.Text>
                                Some quick example text to build on the card title and make up the bulk of
                                the card's content.
                                </Card.Text>
                                <Button variant="primary">Go somewhere</Button>
                            </Card.Body>
                        </Card>
                    </Col>
                </Row>
            </div>
            </>
        );
    };
export default Welcome;