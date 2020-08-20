import React from "react";

import {ThemeProvider} from "styled-components";
import  {useDarkMode} from "./components/useDarkMode"
import { GlobalStyles } from "./components/Globalstyle";
import { lightTheme, darkTheme } from "./components/Themes"
import Toggle from "./components/Toggler"
import Header from './components/Navbar';
import Welcome from "./components/Welcome";

const App= () => {
  
  const [theme, themeToggler, mountedComponent] = useDarkMode();

  const themeMode = theme === 'light' ? lightTheme : darkTheme;

  if(!mountedComponent) return <div/>
  

  return (
    <ThemeProvider theme={themeMode}>
      <>
        <GlobalStyles/>
          <Header theme={theme} />
          <Welcome />
          <Toggle theme={theme} toggleTheme={themeToggler} />
      </>
    </ThemeProvider>
  );
};

export default App;
