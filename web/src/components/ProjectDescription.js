import React from "react";
import { Tabs, Tab } from "react-bootstrap";
import { useParams } from "react-router-dom";
import styled from "styled-components";

const Top = styled.div`
  height: 6rem;
`;

const ProjectDescription = ({ theme, toggleTheme }) => {
  let { projectid } = useParams();
  console.log(projectid);
  return (
    <>
      <Top>
        <h2>Project #{projectid}</h2>
        <h5>Here is a desciption of some stuff...</h5>
      </Top>
      <Tabs defaultActiveKey="overview" transition={false}>
        <Tab eventKey="overview" title="Overview"></Tab>
        <Tab eventKey="input" title="Input Data"></Tab>
        <Tab eventKey="output" title="Output Results"></Tab>
      </Tabs>
    </>
  );
};
export default ProjectDescription;
