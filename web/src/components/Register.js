import React, { useState } from "react";
import { Container, Row, Col, Form, Navbar } from "react-bootstrap";
import styled from "styled-components";
import { PrimaryButton } from "./Buttons";
import cookies from "./../Auth";
import { Redirect } from "react-router-dom";
import { OutlinedPrimaryButton } from "./Buttons";
import Toggle from "./Toggler";
import MemoLogoImage from "../icons/LogoImage.js";
import { FcGoogle } from "react-icons/fc";
import { FaGithub } from "react-icons/fa";
import axios from "axios";

const ClearHeaderBar = styled(Navbar)`
  min-height: 4rem;
  transition: all 0.25s linear;
`;

const LinksRow = styled(Row)`
  font-size: 2.5rem;
`;

const Padding = styled.div`
  padding: 0 0.5rem;
`;

const Text = styled.div`
  font-size: 1rem;
  padding: 0.5rem 0;
`;

const Title = styled.div`
  font-size: 2rem;
  padding: 0.5rem 0;
  padding-top: 1rem;
`;

const RegButton = styled(PrimaryButton)`
  width: 15rem;
  padding: 0;
  height: 2rem;
`;

const RegForm = styled(Form)`
  width: 15rem;
`;

const Link = styled.a`
  padding-top: 3rem;
`;

const Register = ({ theme, toggleTheme }) => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [regState, setRegState] = useState(0);

  const handleSubmit = async (evt) => {
    evt.preventDefault();
    if (password === confirmPassword) {
      let response = await axios.post(
        "/api/users/new",
        { email: email, password: password },
        {
          headers: {
            "Content-Type": "application/json",
          },
        }
      );

      response = response.data;
      console.log(response.token);

      if (response.token === "null") {
        setRegState(2);
      } else {
        setRegState(1);
        cookies.set("token", response.token, { path: "/", sameSite: true });
      }
    } else {
      setRegState(3);
    }
  };

  const checkRegState = () => {
    if (regState === 1) {
      console.log("Authenticated");
      return <Redirect to="/dashboard" />;
    } else if (regState === 2) {
      return <p>Something is wrong with the information you have provided</p>;
    } else if (regState === 3) {
      return <p>Passwords don't match</p>;
    }
  };

  return (
    <Container fluid="xl">
      <ClearHeaderBar sticky="top">
        <Toggle theme={theme} toggleTheme={toggleTheme} />
        <ClearHeaderBar.Collapse className="justify-content-end">
          <ClearHeaderBar.Text>Already have an account?</ClearHeaderBar.Text>
          <OutlinedPrimaryButton variant="primary" href="/login">
            SIGN IN
          </OutlinedPrimaryButton>
        </ClearHeaderBar.Collapse>
      </ClearHeaderBar>

      <Col className="justify-content-md-center">
        <Row className="justify-content-md-center">
          <Link href="/">
            <MemoLogoImage theme={theme} />
          </Link>
        </Row>

        <Title>New Sybl Account</Title>

        <Row className="justify-content-md-center">
          <Row className="justify-content-md-center">
            <RegForm onSubmit={handleSubmit}>
              <RegForm.Group controlId="Email">
                <RegForm.Control
                  type="email"
                  placeholder="Enter email"
                  onChange={(e) => setEmail(e.target.value)}
                />
              </RegForm.Group>

              <RegForm.Group controlId="Password">
                <RegForm.Control
                  type="password"
                  placeholder="Password"
                  onChange={(e) => setPassword(e.target.value)}
                />
              </RegForm.Group>
              <RegForm.Group controlId="Password">
                <RegForm.Control
                  type="password"
                  placeholder="Confirm Password"
                  onChange={(e) => setConfirmPassword(e.target.value)}
                />
              </RegForm.Group>
              <Row className="justify-content-md-center">
                <RegButton variant="primary" type="submit">
                  SIGN UP
                </RegButton>
              </Row>
              {checkRegState()}
            </RegForm>
          </Row>
        </Row>
        <Text>Or use one of the following</Text>
        <LinksRow className="justify-content-md-center">
          <FcGoogle />
          <Padding></Padding>
          <FaGithub />
        </LinksRow>
      </Col>
    </Container>
  );
};
export default Register;
