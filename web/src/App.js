import React from "react";

import {ThemeProvider} from "styled-components";
import  {useDarkMode} from "./components/useDarkMode"
import { GlobalStyles } from "./components/Globalstyle";
import { lightTheme, darkTheme } from "./components/Themes"
import Header from "./components/Navbar";
import ClearHeader from "./components/ClearNavbar";
import Welcome from "./components/Welcome";
import Login from "./components/Login";
import Register from "./components/Register";
import Dashboard from "./components/Dashboard";

import { BrowserRouter as Router, Switch, Route } from "react-router-dom";

const App = () => {
  
  const [theme, themeToggler, mountedComponent] = useDarkMode();

  const themeMode = theme === 'light' ? lightTheme : darkTheme;

  if(!mountedComponent) return <div/>
  
  return (
    <Router>
      <ThemeProvider theme={themeMode}>
        <>
          <GlobalStyles/>
            <Switch>
            <Route path="/register">
              <Register theme={theme} toggleTheme={themeToggler}/>
            </Route>
            <Route path="/login">
              <Login theme={theme} toggleTheme={themeToggler}/>
            </Route>
            <Route path="/dashboard">
              <Header theme={theme} toggleTheme={themeToggler}/>
              <Dashboard/>
            </Route>
            <Route path="/">
              <Header theme={theme} toggleTheme={themeToggler}/>
              <Welcome />
            </Route>
          </Switch>
        </>
      </ThemeProvider>
    </Router>
  );
};

export default App;
