import React from "react";
import {
  Container,
  Col,
  Tabs,
} from "react-bootstrap";
import styled from "styled-components";
import DashHeader from "./DashNavbar";
import axios from "axios";

const MaxCol = styled(Col)`
  height: 100vh;
`;

const SideBar = styled(Tabs)``;

const Top = styled.div`
  height: 6rem;
`;


const Upload = ({ theme, toggleTheme }) => {

    let file_reader;

    const handleFileRead = async (e) => {
        const content = file_reader.result;
        console.log(content);

        let response = await axios.post(
            "/api/jobs/new",
            { content: content },
            {
              headers: {
                "Content-Type": "application/json",
              },
            }
          );

        response = response.data;
        console.log(response);

    }

    const handleFileChosen = (file) => {
        file_reader = new FileReader();
        file_reader.onloadend = handleFileRead;
        file_reader.readAsText(file);
    }


  return (
    <>
      <DashHeader theme={theme} toggleTheme={toggleTheme} />
      <Container fluid>
      <input type="file" id="avatar" name="avatar"
        accept=".csv" onChange={(e) => handleFileChosen(e.target.files[0])}/>
      </Container>
    </>
  );
};
export default Upload;
