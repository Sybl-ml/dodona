import React from "react";
import { Container, Row, Col, Tabs, Tab } from 'react-bootstrap';
import styled from "styled-components";
import DashHeader from "./DashNavbar";
import { ItemCard } from "./Cards"
const MaxCol = styled(Col)`
    height:100vh;
`;

const Dashboard = ({theme, toggleTheme}) => {
        return (
            <>
            
            <DashHeader theme={theme} toggleTheme={toggleTheme}/>
            <Container fluid>
                <Row>
                    <MaxCol xs={{span:12, order: 'last'}} lg={{span:4, order: 'first'}} xl={3}>
                        <Tabs defaultActiveKey="home" transition={false}>
                            <Tab eventKey="projects" title="Projects">
                                <Row>
                                    <ItemCard><h1>Hi</h1></ItemCard>
                                </Row>
                            </Tab>
                            <Tab eventKey="models" title="Models">

                            </Tab>
                        </Tabs>
                    </MaxCol>
                    <MaxCol xs={12} lg={8} xl={9} style={{backgroundColor: "cyan"}}>
                        2
                    </MaxCol>
                </Row>
            </Container>
            </>
        );
    };
export default Dashboard;