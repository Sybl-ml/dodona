import React from "react";
import {ThemeProvider} from "styled-components";

import  {useDarkMode} from "./components/useDarkMode"
import { GlobalStyles } from "./components/Globalstyle";
import { lightTheme, darkTheme } from "./components/Themes"
import Toggle from "./components/Toggler"

const App= () => {
  
  const [theme, themeToggler, mountedComponent] = useDarkMode();

  const themeMode = theme === 'light' ? lightTheme : darkTheme;

  if(!mountedComponent) return <div/>
  

  return (
    <ThemeProvider theme={themeMode}>
      <>
        <GlobalStyles/>
          <div className="App">
            <header className="App-header">
              
            <br/>
              <br/>
              <br/>
              <br/>
              <h1>
                Welcome to Sybl
              </h1>
              <h2>Distributed ML with Ensemble Methods</h2>
              
            </header>

          <Toggle theme={theme} toggleTheme={themeToggler} />
        </div>
      </>
    </ThemeProvider>
  );
};

export default App;
