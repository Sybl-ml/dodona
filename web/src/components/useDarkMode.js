import { useEffect, useState } from 'react';
export const useDarkMode = () => {
    const [theme, setTheme] = useState('light');
    const [mountedComponent, setMountedComponent] = useState(false)
    const setMode = mode => {
        window.localStorage.setItem('theme', mode)
        setTheme(mode)
    };

    const themeToggler = () => {
        const favicon = document.getElementById('favicon')
        if (theme === 'light') {
            setMode('dark')
            favicon.href = "favicon_dark.ico"
        }
        else {
            setMode('light')
            favicon.href = "favicon_light.ico"
        }

    };

    useEffect(() => {
        const localTheme = window.localStorage.getItem('theme');

        const favicon = document.getElementById('favicon')
        setMountedComponent(true)
        if (localTheme === 'light') {
            setTheme(localTheme)
            favicon.href = "favicon_dark.ico"
        }
        else {
            setMode('light')
            favicon.href = "favicon_light.ico"
        }

    }, []);

    return [theme, themeToggler, mountedComponent]
};