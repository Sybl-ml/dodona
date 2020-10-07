import React from "react";
import {
    Container,
    Row,
    Col,
    Tabs,
    Tab,
    InputGroup,
    FormControl,
} from "react-bootstrap";
import styled from "styled-components";

const Top = styled.div`
  height: 6rem;
`;

const ProjectDescription = ({ theme, toggleTheme, match }) => {
    console.log(match.params.projectid)
    return (
        <>
            <Top>
                <h2>Project #3</h2>
                <h5>Here is a desciption of some stuff...</h5>
            </Top>
            <Tabs defaultActiveKey="overview" transition={false}>
                <Tab eventKey="overview" title="Overview"></Tab>
                <Tab eventKey="input" title="Input Data"></Tab>
                <Tab eventKey="output" title="Output Results"></Tab>
            </Tabs>
        </>
    )
}
export default ProjectDescription