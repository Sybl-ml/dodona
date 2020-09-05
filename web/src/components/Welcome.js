import React from "react"
import { Container, Row, Col, Button, Card, } from 'react-bootstrap';
import styled from "styled-components";
import { FaExternalLinkAlt } from 'react-icons/fa';
import { SiTensorflow, SiKeras } from 'react-icons/si';
import MemoPlaceholder from '../icons/Placeholder.js';
import {PrimaryButton, OutlinedPrimaryButton} from './Buttons';

const Main = styled(Row)`
    text-align:center;
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

const FixButton = styled(PrimaryButton)`
    width: auto;
    margin: 1rem;
`;

const OutlinedFixButton = styled(OutlinedPrimaryButton)`
    width: auto;
    margin: 1rem;
`;

const Highlight = styled.div`
    padding-bottom:2rem;
    text-align: center;
    background-color: ${({ theme }) => theme.highlight};
    color: ${({ theme }) => theme.accent};
`;

const FixedRow = styled(Row)`
    margin: 0;
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
                                
                                Works with <a href="https://www.tensorflow.org/"><SiTensorflow /></a> & <a href="https://keras.io/"><SiKeras /></a>
                            </SubTitle>
                        </Row>
                        <Row className="justify-content-md-center">
                            <FixButton variant="primary">GET STARTED</FixButton>
                            <OutlinedFixButton variant="primary" className="outline">
                                <FaExternalLinkAlt /> EXAMPLE
                            </OutlinedFixButton>      
                        </Row>
                    </Col>
                    <Col>
                        <MemoPlaceholder width="120%"/>
                    </Col>
                </Main>
            </Container>

            <Highlight >
                <Quote>TRUSTED BY MANY ACROSS THE GLOBE</Quote>
                <FixedRow className="justify-content-md-center">
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
                </FixedRow>
            </Highlight>
            </>
        );
    };
export default Welcome;