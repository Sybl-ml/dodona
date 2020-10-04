import React from 'react'
import { func, string } from 'prop-types';
import DarkModeToggle from "react-dark-mode-toggle";


const Toggle = ({theme,  toggleTheme}) => {
    return (
      <DarkModeToggle 
        onChange={toggleTheme}
        checked={theme === "light" ? false : true}
        size={90}
      />
    );
};

Toggle.propTypes = {
    theme: string.isRequired,
    toggleTheme: func.isRequired,
};

export default Toggle;