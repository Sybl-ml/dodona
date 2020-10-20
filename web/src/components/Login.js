import React, { useState } from "react";
import { Container, Row, Col, Form, Navbar } from "react-bootstrap";
import { Redirect } from "react-router-dom";
import styled from "styled-components";
import { PrimaryButton } from "./Buttons";
import cookies from "./../Auth";
import MemoLogoImage from "../icons/LogoImage.js";
import { FcGoogle } from "react-icons/fc";
import { FaGithub } from "react-icons/fa";
import { OutlinedPrimaryButton } from "./Buttons";
import Toggle from "./Toggler";
import axios from "axios";

const ClearHeaderBar = styled(Navbar)`
  min-height: 4rem;
  transition: all 0.25s linear;
  background: none;
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

const LoginButton = styled(PrimaryButton)`
  width: 15rem;
  padding: 0;
  height: 2rem;
`;

const LoginForm = styled(Form)`
  width: 15rem;
`;

const Forgot = styled.a`
  font-size: 0.8rem;
  padding-bottom: 0;
  padding-top: 0.8rem;
`;

const Link = styled.a`
  padding-top: 3rem;
`;

const Login = ({ theme, toggleTheme }) => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [, setRemember] = useState("");
  const [loginState, setLoginState] = useState(0);

  const handleSubmit = async (evt) => {
    evt.preventDefault();
    let response = await axios.post(
      "/api/users/login",
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
      setLoginState(2);
    } else {
      setLoginState(1);
      console.log(response.token);
      cookies.set("token", response.token, { path: "/", sameSite: true });
    }
  };

  const checkLoginState = () => {
    if (loginState === 1) {
      console.log("Authenticated");
      return <Redirect to="/dashboard" />;
    } else if (loginState === 2) {
      console.log("Not Authenticated");
      return <p>Something is wrong with your login information</p>;
    }
  };

  return (
    <Container>
      <ClearHeaderBar collapseOnSelect expand="sm" sticky="top">
        <ClearHeaderBar.Toggle
          aria-controls="responsive-navbar-nav"
          className="justify-content-end styled-toggle"
        />
        <ClearHeaderBar.Collapse>
          <Toggle theme={theme} toggleTheme={toggleTheme} />
        </ClearHeaderBar.Collapse>
        <ClearHeaderBar.Collapse className="justify-content-end">
          <ClearHeaderBar.Text>Don't have an account?</ClearHeaderBar.Text>
          <OutlinedPrimaryButton variant="primary" href="/register">
            CREATE ONE
          </OutlinedPrimaryButton>
        </ClearHeaderBar.Collapse>
      </ClearHeaderBar>

      <Col className="justify-content-center">
        <Row className="justify-content-center">
          <Link href="/">
            <MemoLogoImage theme={theme} />
          </Link>
        </Row>

        <Row className="justify-content-center">
          <Title>Sign In To Sybl</Title>
        </Row>
        <Row className="justify-content-center">
          <Row className="justify-content-center">
            <LoginForm onSubmit={handleSubmit}>
              <LoginForm.Group controlId="Email">
                <LoginForm.Control
                  type="email"
                  placeholder="Enter email"
                  onChange={(e) => setEmail(e.target.value)}
                />
              </LoginForm.Group>
              <LoginForm.Group controlId="Password">
                <LoginForm.Control
                  type="password"
                  placeholder="Password"
                  onChange={(e) => setPassword(e.target.value)}
                />
              </LoginForm.Group>
              <LoginForm.Group controlId="RememberMe">
                <LoginForm.Check
                  type="checkbox"
                  label="Remember Me"
                  onChange={(e) => setRemember(e.target.value)}
                />
              </LoginForm.Group>
              <Row className="justify-content-center">
                <LoginButton variant="primary" type="submit">
                  SIGN IN
                </LoginButton>
              </Row>
              <Row className="justify-content-center">
                <Forgot href="/forgot">Forgotten Password?</Forgot>
              </Row>
              {checkLoginState()}
            </LoginForm>
          </Row>
        </Row>
        <Text>Or continue with</Text>
        <LinksRow className="justify-content-center">
          <FcGoogle />
          <Padding></Padding>
          <FaGithub />
        </LinksRow>
      </Col>
    </Container>
  );
};
export default Login;