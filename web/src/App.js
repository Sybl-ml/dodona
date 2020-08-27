import React from "react";

import {ThemeProvider} from "styled-components";
import  {useDarkMode} from "./components/useDarkMode"
import { GlobalStyles } from "./components/Globalstyle";
import { lightTheme, darkTheme } from "./components/Themes"
import Toggle from "./components/Toggler"
import Header from "./components/Navbar";
import ClearHeader from "./components/ClearNavbar";
import Welcome from "./components/Welcome";
import Login from "./components/Login";
import Register from "./components/Register";

import { BrowserRouter as Router, Switch, Route } from "react-router-dom";

const App= () => {
  
  const [theme, themeToggler, mountedComponent] = useDarkMode();

  const themeMode = theme === 'light' ? lightTheme : darkTheme;

  if(!mountedComponent) return <div/>

  function submit(event) {
    console.log(event.target.elements.Email.value);
    console.log(event.target.elements.Password.value);
    console.log(event.target.elements.RememberMe.value);
};
  

  return (
    <Router>
    <ThemeProvider theme={themeMode}>
      <>
        <GlobalStyles/>
        
          <Switch>
          <Route path="/register">
            <ClearHeader theme={theme} />
            <Register />
            <Toggle theme={theme} toggleTheme={themeToggler} />
          </Route>
          <Route path="/login">
            <ClearHeader theme={theme} />
            <Login/>
            <Toggle theme={theme} toggleTheme={themeToggler} />
          </Route>
          <Route path="/">
            <Header theme={theme} />
            <Welcome />
            <Toggle theme={theme} toggleTheme={themeToggler} />
          </Route>
        </Switch>
      </>
    </ThemeProvider>
    </Router>
  );
};

export default App;
