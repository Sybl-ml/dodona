import React from "react";
import { Container, Row, Col, Tabs, Tab } from 'react-bootstrap';
import styled from "styled-components";
import DashHeader from "./DashNavbar";
import { ProjectCard, ModelCard } from "./Cards"

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
                        <Tabs defaultActiveKey="home" transition={false} defaultActiveKey="projects">
                            <Tab eventKey="projects" title="Projects">
                                <ProjectCard 
                                    title="Project #6"
                                    time = "7 Days"
                                />
                                <ProjectCard 
                                    title="Project #10"
                                    time = "1 Hour"
                                />
                                <ProjectCard 
                                    title="Project #8"
                                    time = "30 Mins"
                                />
                                <ProjectCard 
                                    title="Project #2"
                                    time = "10 Hours"
                                />
                            </Tab>
                            <Tab eventKey="models" title="Models">
                                <ModelCard 
                                    title="Cool Model 2"
                                />
                                <ModelCard 
                                    title="Best Model XD"
                                />
                                <ModelCard 
                                    title="This one is bad haha"
                                />
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