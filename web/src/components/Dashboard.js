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
import DashHeader from "./DashNavbar";
import { ProjectCard, ModelCard } from "./Cards";
import { FaSearch } from "react-icons/fa";

const MaxCol = styled(Col)`
  height: 100vh;
`;

const SideBar = styled(Tabs)``;

const Top = styled.div`
  height: 6rem;
`;
const Dashboard = ({ theme, toggleTheme }) => {
  return (
    <>
      <DashHeader theme={theme} toggleTheme={toggleTheme} />
      <Container fluid>
        <Row>
          <MaxCol
            xs={{ span: 12, order: "last" }}
            lg={{ span: 3, order: "first" }}
          >
            <Top>
              <InputGroup className="mb-2" style={{ padding: "1.5rem 0" }}>
                <FormControl
                  id="inlineFormInputGroup"
                  placeholder="Search"
                ></FormControl>
              </InputGroup>
            </Top>
            <SideBar
              defaultActiveKey="home"
              transition={false}
              defaultActiveKey="projects"
            >
              <Tab eventKey="projects" title="Projects">
                <ProjectCard title="Project #6" time="7 Days" />
                <ProjectCard title="Project #10" time="1 Hour" />
                <ProjectCard title="Project #8" time="30 Mins" />
                <ProjectCard title="Project #2" time="10 Hours" />
              </Tab>
              <Tab eventKey="models" title="Models">
                <ModelCard title="Cool Model 2" />
                <ModelCard title="Best Model XD" />
                <ModelCard title="This one is bad haha" />
              </Tab>
            </SideBar>
          </MaxCol>
          <Col xs={12} lg={9} style={{ textAlign: "left" }}>
            <Top>
              <h2>Project #3</h2>
              <h5>Here is a desciption of some stuff...</h5>
            </Top>
            <Tabs defaultActiveKey="overview" transition={false}>
              <Tab eventKey="overview" title="Overview"></Tab>
              <Tab eventKey="input" title="Input Data"></Tab>
              <Tab eventKey="output" title="Output Results"></Tab>
            </Tabs>
          </Col>
        </Row>
      </Container>
    </>
  );
};
export default Dashboard;